#![feature(proc_macro_def_site)]
#![feature(proc_macro_span)]

use proc_macro::{TokenStream};
use proc_macro2::Span;

#[proc_macro_attribute]
pub fn node_export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);
    let span = proc_macro::Span::call_site();
    dbg!(span);
    let span = dbg!(span.located_at(proc_macro::Span::def_site()));
    let tokens = quote::quote_spanned! {
        Span::from(span) =>
        #[cfg_attr(feature = "node", napi_derive::napi(#attr))]
        #item
    };
    println!("{tokens}");
    proc_macro::TokenStream::from(tokens)
}

#[proc_macro_attribute]
pub fn node_only(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);
    let tokens = quote::quote! {
        #[cfg(feature = "node")]
        #[napi(#attr)]
        #item
    };
    proc_macro::TokenStream::from(tokens)
}
