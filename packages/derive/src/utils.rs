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
    let mut result: Vec<&syn::Type> = vec![];
    for arg in &func_sig.inputs {
        match arg {
            syn::FnArg::Receiver(receiver) => match receiver.reference {
                Some(_) => continue,
                None => abort_by!(arg, by, "non reference receiver is not allowed."),
            },
            syn::FnArg::Typed(arg_info) => match arg_info.ty.as_ref() {
                syn::Type::BareFn(_) => {
                    abort_by!(arg, by, "function type by parameter is not allowed.")
                }
                syn::Type::Reference(_) => {
                    abort_by!(arg, by, "reference type by parameter is not allowed.")
                }
                syn::Type::Ptr(_) => {
                    abort_by!(arg, by, "ptr type by parameter is not allowed.")
                }
                _ => result.push(arg_info.ty.as_ref()),
            },
        }
    }
    result
}

pub fn has_return_value(return_type: &syn::ReturnType) -> bool {
    match return_type {
        syn::ReturnType::Default => false,
        syn::ReturnType::Type(_, _) => true,
    }
}

pub fn make_typed_return(return_type: &syn::ReturnType) -> TokenStream {
    if has_return_value(return_type) {
        quote! { -> u32 }
    } else {
        quote! {}
    }
}
