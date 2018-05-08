#![feature(proc_macro)]
#![recursion_limit = "256"]

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
use syn::synom::{Synom, Parser};
use syn::{Expr, Ident, ItemFn, Local, Pat, Stmt, ReturnType, FnArg, Type};
use syn::token::Comma;

#[proc_macro_attribute]
pub fn kernel(args: TokenStream, input: TokenStream) -> TokenStream {
    // Return the input unchanged if it failed to parse. The compiler will show
    // the right diagnostics.
    let input: ItemFn = match syn::parse(input.clone()) {
        Ok(input) => input,
        Err(_) => return input,
    };

    assert!(args.is_empty(), "Kernel attribute accepts no arguments");

    let mut kernel_args = Vec::new();
    for arg in input.decl.inputs.iter() {
        kernel_args.push((*arg).clone());
    }

    match input.decl.output.clone() {
        ReturnType::Default => panic!("Kernels must return a value"), // or do they?
        ReturnType::Type(_, ty) => match *ty {
            Type::Tuple(..) => {

            },
            _ => kernel_args.push(FnArg::Captured(ArgCaptured {
                pat: Pat::Verbatim(PatVerbatim {
                    tts: quote!()
                }),
                colon_token: Token![:],
                ty: 
            })),
        }
    }

    quote!(
        #input.vis fn #input.ident (  )
        
    ).into()
}

#[proc_macro]
pub fn dispatch(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated_nonempty;
    let mut args = match parser.parse(input.clone()) {
        Ok(input) => input,
        Err(_) => return input,
    }; 

    let kernel = args.pop().expect("Dispatch requires arguments").into_value();

    for expr in args.iter() {
        match expr {
            Expr::Reference(_) => {},
            _ => panic!("Dispatch arguments must be references: {:?}", expr.into_tokens()),
        }
    }

    quote!( {
        let f = #kernel ;
        f ( #(#args),* );
    } ).into()
}
