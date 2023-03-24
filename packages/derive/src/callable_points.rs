use proc_macro2::Span;
use proc_macro2::TokenTree;
use quote::quote;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::callable_point;

pub fn make_callable_points(body: Vec<syn::Item>) -> (Vec<TokenTree>, Vec<(String, bool)>) {
    let mut list_callable_points = Vec::new();
    let mut made = Vec::new();

    for i in &body {
        if let syn::Item::Fn(function) = i.clone() {
            let (function_remove_macro, is_stripped) = strip_callable_point(function);

            if is_stripped {
                let (made_callable_point, list_callable_point) =
                    callable_point::make_callable_point(function_remove_macro);
                made.extend(made_callable_point);
                list_callable_points.push(list_callable_point);
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
/// The second return value is the result of whether or not `#[callable_point]` was found.
/// This is used to create a callable point.
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

/// Returns a literal of list_callable_points.
///
/// This is for using serialized binaries in `#[callable_points]`.
pub fn make_callee_map_lit(list_callable_points: Vec<(String, bool)>) -> syn::LitByteStr {
    #[derive(Serialize, Deserialize)]
    struct CalleeProperty {
        is_read_only: bool,
    }
    #[derive(Serialize, Deserialize)]
    struct CalleeMap {
        #[serde(flatten)]
        inner: HashMap<String, CalleeProperty>,
    }

    let mut inner_callee_map = HashMap::new();

    for callable_point in list_callable_points.into_iter() {
        let callee_info = CalleeProperty {
            is_read_only: callable_point.1,
        };
        inner_callee_map.insert(callable_point.0, callee_info);
    }

    let callee_map: CalleeMap = CalleeMap {
        inner: inner_callee_map,
    };

    let callee_map_vec = serde_json::to_vec(&callee_map).unwrap();
    syn::LitByteStr::new(&callee_map_vec[..], Span::call_site())
}
