use proc_macro2::Span;
use proc_macro2::TokenTree;
use quote::quote;
use serde::ser::{Serialize, SerializeMap, Serializer};

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

pub fn make_callee_map_lit(list_callable_points: Vec<(String, bool)>) -> syn::LitByteStr {
    struct CalleeMap<K, V> {
        inner: Vec<(K, V)>,
    }

    impl<K, V> Serialize for CalleeMap<K, V>
    where
        K: Serialize,
        V: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(self.inner.len()))?;
            for (k, v) in &self.inner {
                map.serialize_entry(&k, &v)?;
            }
            map.end()
        }
    }

    let callee_map: CalleeMap<String, bool> = CalleeMap {
        inner: list_callable_points,
    };

    let callee_map_vec = serde_json::to_vec_pretty(&callee_map).unwrap();
    syn::LitByteStr::new(&callee_map_vec[..], Span::call_site())
}
