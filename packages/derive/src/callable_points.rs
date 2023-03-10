use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::Item;

use crate::callable_point;

pub fn make_callable_points(body: Vec<Item>) -> (Vec<TokenTree>, Vec<TokenStream>) {
    let mut list_callable_points = Vec::new();
    let mut made = Vec::new();

    for i in &body {
        if let syn::Item::Fn(function) = i.clone() {
            let (function_remove_macro, is_stripped) = strip_callable_point(function.clone());

            if is_stripped {
                let (made_callable_point, callee_func) =
                    callable_point::make_callable_point(function_remove_macro);
                made.extend(made_callable_point);
                list_callable_points.push(callee_func);
            } else {
                made.extend(quote! {#i});
            }
        } else {
            made.extend(quote! {#i});
        }
    }

    (made, list_callable_points)
}

/// If the function has `#[callable_point]`, strip it.
///
/// example:
///
/// ```no_test
/// #[callable_point]
/// fn foo(deps: Deps, _env: Env, ...) -> u32 {
///     42
/// }
/// ```
///
/// Output:
///
/// ```no_test
/// fn foo(deps: Deps, _env: Env, ...) -> u32 {
///     42
/// }
/// ```
pub fn strip_callable_point(function: syn::ItemFn) -> (syn::ItemFn, bool) {
    let mut attrs_stripped = vec![];
    let mut is_stripped = false;
    for attr in function.attrs {
        if attr.path.is_ident("callable_point") {
            is_stripped = true;
        } else {
            attrs_stripped.push(attr)
        }
    }

    (
        syn::ItemFn {
            attrs: attrs_stripped,
            vis: function.vis,
            sig: function.sig,
            block: function.block,
        },
        is_stripped,
    )
}
