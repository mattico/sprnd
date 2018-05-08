#![feature(proc_macro)]
#![recursion_limit = "256"]

extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::BTreeMap;
use syn::punctuated::Punctuated;
use syn::synom::Parser;
use syn::*;

// Turns a kernel arg into the generated slice arg type
fn make_arg(ty_map: &mut BTreeMap<Ident, Ident>, arg: &FnArg, count: u32) -> FnArg {
    let arg = match arg {
        FnArg::Captured(cap) => cap,
        _ => panic!("Kernel function argument was of incorrect type"),
    };
    let ty = Box::new(arg.ty.clone());
    let slice_ty = Box::new(Type::Slice(TypeSlice {
        bracket_token: token::Bracket(Span::call_site()),
        elem: ty,
    }));

    let old_ident = match arg.pat {
        Pat::Ident(ref ident) => ident.ident.clone(),
        _ => panic!("Kernel fuction argument used invalid pattern"),
    };
    let ident = Ident::new(&format!("{}{}", "in", count), Span::call_site());
    ty_map.insert(old_ident, ident);

    let ty = Type::Reference(TypeReference {
        and_token: token::And([Span::call_site()]),
        lifetime: None,
        mutability: None,
        elem: slice_ty,
    });

    let pat = Pat::Ident(PatIdent {
        by_ref: None,
        mutability: None,
        ident,
        subpat: None,
    });

    FnArg::Captured(ArgCaptured {
        pat,
        colon_token: token::Colon([Span::call_site()]),
        ty,
    })
}

// Turns a return value into the generated slice arg type
fn make_return_arg(ty: &Type, count: u32) -> FnArg {
    let ty = Box::new((*ty).clone());
    let slice_ty = Box::new(Type::Slice(TypeSlice {
        bracket_token: token::Bracket(Span::call_site()),
        elem: ty,
    }));

    let ident = Ident::new(&format!("out{}", count), Span::call_site());

    let ty = Type::Reference(TypeReference {
        // There's gotta be a better way...
        and_token: token::And([Span::call_site()]),
        lifetime: None,
        mutability: Some(token::Mut(Span::call_site())),
        elem: slice_ty,
    });

    let pat = Pat::Ident(PatIdent {
        by_ref: None,
        mutability: None,
        ident,
        subpat: None,
    });

    FnArg::Captured(ArgCaptured {
        pat,
        colon_token: token::Colon([Span::call_site()]),
        ty,
    })
}

#[proc_macro_attribute]
pub fn kernel(args: TokenStream, input: TokenStream) -> TokenStream {
    // Return the input unchanged if it failed to parse. The compiler will show
    // the right diagnostics.
    let input: ItemFn = match syn::parse(input.clone()) {
        Ok(input) => input,
        _ => return input,
    };

    assert!(args.is_empty(), "Kernel attribute accepts no arguments");

    // TODO: Fix this madness
    let mut arg_renaming = BTreeMap::new();

    let mut kernel_args = Vec::new();
    let mut kernel_rets = Vec::new();
    let mut in_count = 0;
    let mut out_count = 0;
    for arg in input.decl.inputs.iter() {
        kernel_args.push(make_arg(&mut arg_renaming, arg, in_count));
        in_count += 1;
    }

    match input.decl.output.clone() {
        ReturnType::Default => panic!("Kernels must return a value"), // or do they?
        ReturnType::Type(_, ty) => /*match *ty {
            Type::Tuple(tup) => {
                for elem in tup.elems.iter() {
                    kernel_args.push(make_return_arg(elem, out_count));
                    out_count += 1;
                }
            }
            _ => kernel_args.push(make_return_arg(&ty, out_count)),
        },*/
        kernel_rets.push(make_return_arg(&*ty, out_count)),
    }

    let args = kernel_args.iter().chain(kernel_rets.iter());
    let arg_pats: Vec<_> = args
        .iter()
        .map(|arg| match arg {
            FnArg::Captured(cap) => cap.pat.clone(),
            _ => panic!("Invalid kernel arg type"),
        })
        .collect();
    let first_arg_pat = arg_pats.iter().next().clone();
    let arg_lets: Vec<_> = kernel_args
        .iter()
        .map(|arg| match arg {
            FnArg::Captured(ref cap) => cap.ty.clone(),
            _ => panic!("Invalid kernel arg type"),
        })
        .collect();
    let ret_lets: Vec<_> = kernel_rets
        .iter()
        .map(|arg| match arg {
            
        })
        .collect();
    let body = input.block.stmts.clone();
    let ItemFn {
        vis,
        unsafety,
        abi,
        ident,
        ..
    } = input;

    let tokens = quote!{
        #vis #unsafety #abi fn #ident( #(#args),* ) {
            assert!( #(#arg_pats .len())==* );
            for i in 0.. #first_arg_pat .len() {
                #(#arg_lets);*
                #(#ret_lets);*
                #ret_pat = { #(#body)* };
            }
        }
    };

    panic!("{:?}", tokens);

    tokens.into()
}

#[proc_macro]
pub fn dispatch(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated_nonempty;
    let mut args = match parser.parse(input.clone()) {
        Ok(input) => input,
        Err(_) => return input,
    };

    let kernel = args.pop()
        .expect("Dispatch requires arguments")
        .into_value();

    for expr in args.iter() {
        match expr {
            Expr::Reference(_) => {}
            _ => panic!(
                "Dispatch arguments must be references: {:?}",
                expr.into_tokens()
            ),
        }
    }

    quote!( {
        let f = #kernel ;
        f ( #(#args),* );
    } ).into()
}
