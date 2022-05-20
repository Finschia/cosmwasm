use proc_macro2::TokenStream;
use quote::quote;

macro_rules! abort_by {
    ($span:expr, $by:expr, $($tts:tt)*) => {
        proc_macro_error::abort!($span, $($tts)*;
        note = format!("this error originates in the attribute macro `{}`", $by)
    )
    };
}
// it's for cannot use #[macro_export] with proc_macro
// but, It occured false-positive by clippy, so avoid it.
// https://github.com/rust-lang/rust-clippy/issues/1938
#[cfg_attr(feature = "cargo-clippy", allow(clippy::useless_attribute))]
#[allow(clippy::single_component_path_imports)]
pub(crate) use abort_by;

pub fn collect_available_arg_types(func_sig: &syn::Signature, by: String) -> Vec<&syn::Type> {
    func_sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => abort_by!(arg, by, "method type is not allowed."),
            syn::FnArg::Typed(arg_info) => match arg_info.ty.as_ref() {
                syn::Type::BareFn(_) => {
                    abort_by!(arg, by, "function type by parameter is not allowed.")
                }
                syn::Type::Reference(_) => {
                    abort_by!(arg, by, "reference type by parameter is not allowed.")
                }
                syn::Type::Ptr(_) => abort_by!(arg, by, "Ptr type by parameter is not allowed."),
                _ => arg_info.ty.as_ref(),
            },
        })
        .collect()
}

pub fn get_return_len(return_type: &syn::ReturnType) -> usize {
    match return_type {
        syn::ReturnType::Default => 0,
        syn::ReturnType::Type(_, return_type) => match return_type.as_ref() {
            syn::Type::Tuple(tuple) => tuple.elems.len(),
            _ => 1,
        },
    }
}

pub fn make_typed_return(return_type: &syn::ReturnType, by: String) -> TokenStream {
    let return_types_len = get_return_len(return_type);
    match return_types_len {
        0 => quote! {},
        _ => quote! { -> u32 },
    }
}
