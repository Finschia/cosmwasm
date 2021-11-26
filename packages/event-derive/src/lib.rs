extern crate proc_macro;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, DataStruct, DeriveInput, Error, Field, Ident, Result};

/// scan attrs and get `to_string_fn` attribute
fn scan_to_string_fn(field: Field) -> Result<Option<proc_macro2::TokenStream>> {
    let filtered: Vec<&Attribute> = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("to_string_fn"))
        .collect();
    if filtered.len() > 1 {
        return Err(Error::new(
            field.span(),
            "[IntoEvent] Only one or zero `to_string_fn` can be applied to one field.",
        ));
    };
    if filtered.is_empty() {
        Ok(None)
    } else {
        Ok(Some(filtered[0].tokens.clone()))
    }
}

/// generate an ast for `impl Into<cosmwasm::Event>` from a struct
///
/// Structure:
///
/// ```no_test
/// #[derive(IntoEvent)]
/// struct StructName {
///     field_name_1: field_type_1,
///     field_name_2: field_type_2,
///     // if the value's type does not implement `ToString` trait,
///     // programmers need specify the function with `to_string_fn`
///     // attribute.
///     // this `cast_fn_3` needs to have type `field_type -> String`.
///     #[to_string_fn(cast_fn_3)]
///     field_name_3: field_type_3,
/// }
/// ```
///
/// Output AST:
///
/// ```no_test
/// impl Into<cosmwasm::Event> for `StructName` {
///     fn into(self) -> Event {
///         Event::new("struct_name")
///             .add_attribute("field_name_1", field_value_1)
///             .add_attribute("field_name_2", field_value_2)
///             .add_attribute("field_name_3", cast_fn_3(field_value_3))
///     }
/// }
/// ```
fn make_init(id: Ident, struct_data: DataStruct) -> Result<proc_macro2::TokenStream> {
    // snake case of struct ident
    let name = id.to_string().as_str().to_case(Case::Snake);

    // generate the body of `fn into`
    // generating `Event::new()` part
    let mut fn_body = quote!(
        cosmwasm_std::Event::new(#name)
    );

    // chain `.add_attribute`s to `Event::new()` part
    for field in struct_data.fields {
        let key = match field.clone().ident {
            None => {
                return Err(Error::new(
                    field.span(),
                    "[IntoEvent] Unexpected unnamed field.",
                ))
            }
            Some(id) => id,
        };
        let value = match scan_to_string_fn(field)? {
            Some(to_string_fn) => quote!(#to_string_fn(self.#key)),
            None => quote!(self.#key),
        };
        fn_body.extend(quote!(
            .add_attribute(stringify!(#key), #value)
        ))
    }

    // generate the `impl Into<cosmwasm_std::Event>` from generated `fn_body`
    let gen = quote!(
        impl Into<cosmwasm_std::Event> for #id {
            fn into(self) -> cosmwasm_std::Event {
                #fn_body
            }
        }
    );
    Ok(gen)
}

#[proc_macro_derive(IntoEvent, attributes(to_string_fn))]
pub fn derive_into_event(input: TokenStream) -> TokenStream {
    // parse the input structure into `ItemStruct`
    let derive_input = parse_macro_input!(input as DeriveInput);
    match derive_input.data {
        syn::Data::Struct(struct_data) => make_init(derive_input.ident, struct_data)
            .unwrap_or_else(|e| e.to_compile_error())
            .into(),
        syn::Data::Enum(enum_data) => Error::new(
            enum_data.enum_token.span,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Enum.",
        )
        .to_compile_error()
        .into(),
        syn::Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "[IntoEvent] `derive(IntoEvent)` cannot be applied to Union.",
        )
        .to_compile_error()
        .into(),
    }
}
