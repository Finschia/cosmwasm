use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Error, Fields, Ident, Result};

fn generate_get_address(address_field: &Ident) -> TokenStream {
    quote! {
        fn get_address(&self) -> cosmwasm_std::Addr {
            self.#address_field.clone()
        }
    }
}

fn generate_set_address(address_field: &Ident) -> TokenStream {
    quote! {
        fn set_address(&mut self, address: cosmwasm_std::Addr) {
            self.#address_field = address
        }
    }
}

fn generate_impl_contract(struct_id: &Ident, address_field: &Ident) -> TokenStream {
    let get_fn = generate_get_address(address_field);
    let set_fn = generate_set_address(address_field);
    quote! {
        impl Contract for #struct_id {
            #get_fn
            #set_fn
        }
    }
}

fn has_address_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().filter(|a| a.path.is_ident("address")).count() > 0
}

/// scan fields and extraction a field specifying address.
/// The priority is "contract_address" -> "contract_addr"
/// -> "address" -> "addr"
fn scan_address_field(fields: &Fields) -> Option<Ident> {
    let candidates = vec!["contract_address", "contract_addr", "address", "addr"];
    for field in fields {
        match &field.ident {
            Some(id) => {
                for candidate in &candidates {
                    if id == candidate {
                        return Some(id.clone());
                    }
                }
            }
            None => continue,
        }
    }
    None
}

fn find_address_field_id(fields: Fields) -> Result<Ident> {
    let filtered = fields
        .iter()
        .filter(|field| has_address_attribute(&field.attrs));
    match filtered.clone().count() {
        0 => match scan_address_field(&fields) {
            Some(id) => Ok(id),
            None => Err(Error::new(
                fields.span(),
                "[Contract] There are no field specifying address.",
            )),
        },
        1 => {
            let field = filtered.last().unwrap().clone();
            match field.ident {
                Some(id) => Ok(id),
                None => Err(Error::new(
                    field.span(),
                    "[Contract] The field attributed `address` has no name.",
                )),
            }
        }
        _ => Err(Error::new(
            fields.span(),
            "[Contract] Only one or zero fields can have `address` attribute.",
        )),
    }
}

/// derive `Contract` from a derive input. The input needs to be a struct.
pub fn derive_contract(input: DeriveInput) -> TokenStream {
    match input.data {
        syn::Data::Struct(struct_data) => match find_address_field_id(struct_data.fields) {
            Ok(address_field_id) => generate_impl_contract(&input.ident, &address_field_id),
            Err(e) => e.to_compile_error(),
        },
        syn::Data::Enum(enum_data) => Error::new(
            enum_data.enum_token.span,
            "[Contract] `derive(Contract)` cannot be applied to Enum.",
        )
        .to_compile_error(),
        syn::Data::Union(union_data) => Error::new(
            union_data.union_token.span,
            "[Contract] `derive(Contract)` cannot be applied to Union.",
        )
        .to_compile_error(),
    }
}
