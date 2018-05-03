#![feature(proc_macro)]

extern crate proc_macro;

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use quote::ToTokens;
use std::collections::HashSet as Set;
use syn::fold::{self, Fold};
use syn::punctuated::Punctuated;
use syn::synom::Synom;
use syn::{Expr, Ident, ItemFn, Local, Pat, Stmt};

#[proc_macro_attribute]
pub fn kernel(args: TokenStream, input: TokenStream) -> TokenStream {
    // Return the input unchanged if it failed to parse. The compiler will show
    // the right diagnostics.
    let input: ItemFn = match syn::parse(input.clone()) {
        Ok(input) => input,
        Err(_) => return input,
    };

    assert!(args.is_empty(), "Kernel attribute accepts no arguments");

    quote!(#input).into()
}
