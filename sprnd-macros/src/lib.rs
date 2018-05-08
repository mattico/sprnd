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
use syn::punctuated::Punctuated;
use syn::synom::Parser;
use syn::*;

// Turns a kernel arg into the generated slice arg type
fn make_arg(ty: &Type, count: u32, mutable: bool) -> FnArg {
    let ty = Box::new((*ty).clone());
    let slice_ty = Box::new(Type::Slice(TypeSlice { 
        bracket_token: token::Bracket(Span::call_site()),
        elem: ty,
    }));

    let ty = if mutable {
        Type::Reference(TypeReference {
            // There's gotta be a better way...
            and_token: token::And([Span::call_site()]),
            lifetime: None,
            mutability: Some(token::Mut(Span::call_site())),
            elem: slice_ty,
        })
    } else {
        Type::Reference(TypeReference {
            and_token: token::And([Span::call_site()]),
            lifetime: None,
            mutability: None,
            elem: slice_ty,
        })
    };

    let ident = Ident::new(
        &format!("{}{}", if mutable { "out" } else { "in" }, count), 
        Span::call_site());

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

    let mut kernel_args = Vec::new();
    let mut in_count = 0;
    let mut out_count = 0;
    for arg in input.decl.inputs.iter() {
        match arg {
            FnArg::Captured(c) => kernel_args.push(make_arg(&c.ty, in_count, false)),
            _ => panic!("Kernel function argument was of incorrect type"),
        }
        in_count += 1;
    }

    match input.decl.output.clone() {
        ReturnType::Default => panic!("Kernels must return a value"), // or do they?
        ReturnType::Type(_, ty) => match *ty {
            Type::Tuple(tup) => {
                for elem in tup.elems.iter() {
                    kernel_args.push(make_arg(elem, out_count, true)); 
                    out_count += 1;
                }
            },
            _ => kernel_args.push(make_arg(&ty, out_count, true)),
        }
    }

    let args = &kernel_args;
    let arg_pats: Vec<_> = kernel_args.iter().map(|arg| {
        match arg {
            FnArg::Captured(cap) => cap.pat.clone(),
            _ => panic!("Invalid kernel arg type"),
        }
    }).collect();
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
            #(#body)*
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
