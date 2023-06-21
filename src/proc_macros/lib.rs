// Allow the crate to have a non-snake-case name (touchHLE).
// This also allows items in the crate to have non-snake-case names.
#![allow(non_snake_case)]

use proc_macro;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{self, ReturnType};
use syn::{token, FnArg, Ident, Item, Pat, PatType, Type};
/// IMM: doc
#[proc_macro_attribute]
pub fn boxify(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut output = TokenStream::new();
    match syn::parse::<Item>(item.clone()).unwrap() {
        Item::Fn(mut fn_item) => {
            let mut sig = &mut fn_item.sig;
            assert!(
                sig.asyncness.is_some(),
                "#[boxify] can only be called on async functions!"
            );
            // TODO: Should probably _actually_ enforce this, but I'm not sure how/if that is
            // possible...
            assert!(
                sig.variadic.is_none(),
                "#[boxify] can only be called on rust abi functions! (no variadics!)"
            );
            assert!(
                !sig.inputs.iter().any(|arg| {
                    if let FnArg::Typed(PatType { pat, .. }) = arg {
                        if let Pat::Ident(_) | Pat::Verbatim(_) = **pat {
                            false
                        } else {
                            true
                        }
                    } else {
                        false
                    }
                }),
                "#[boxify] does not (currenty) support destructuring in function args!"
            );
            let new_ret_type = match sig.output {
                ReturnType::Default => {
                    quote::quote!(std::pin::Pin<Box<dyn std::future::Future<Output = ()> + '_>>)
                }
                ReturnType::Type(_, ref ret_type) => {
                    quote::quote!(std::pin::Pin<Box<dyn std::future::Future<Output = #ret_type> + '_>>)
                }
            };

            // IMM: What does this do? (DOC)
            let vis = &fn_item.vis;
            let attrs = &fn_item.attrs;
            let mut boxing_sig = sig.clone();
            let rarrow = token::RArrow::default();
            let arg_names = sig.inputs.iter().filter_map(|arg| {
                if let syn::FnArg::Typed(PatType { pat, .. }) = arg {
                    Some(pat.to_token_stream())
                } else {
                    None
                }
            });

            boxing_sig.asyncness = None;
            boxing_sig.output = ReturnType::Type(rarrow, Box::new(Type::Verbatim(new_ret_type)));

            let name_str = sig.ident.to_string() + "_";
            let name_tok = name_str.parse::<TokenStream>().unwrap();
            sig.ident = Ident::new(&name_str, Span::call_site().into());
            let boxing_fn = if sig.receiver().is_some() {
                quote::quote!(#(#attrs)* #vis #boxing_sig {Box::pin(self.#name_tok(#(#arg_names),*))})
            } else {
                quote::quote!(#(#attrs)* #vis #boxing_sig {Box::pin(#name_tok(#(#arg_names),*))})
            };
            output.extend(quote::quote!(#boxing_fn #fn_item));
        }
        _ => {
            unimplemented!("#[boxify] is not implemented for non-fn types!")
        }
    }
    //todo!("{}", output.to_string());
    output.into()
}
