extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::*;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, token, Ident, Result, Token, FnArg};

#[derive(Copy, Clone, PartialEq, Eq)]
enum PreambleState {
    StructLiteral,
    StructName,
    LoaderLiteral,
    LoaderName,
    Return,
}

#[derive(Debug)]
struct Preamble {
    pointer_struct_name: proc_macro::Ident,
    loader_function_name: proc_macro::Ident,
}

impl Preamble {
    fn new(tokens: &mut dyn Iterator<Item=TokenTree>) -> Self {
        let mut state = PreambleState::StructLiteral;

        let mut pointer_struct_name = None;
        let mut loader_function_name = None;
        loop {
            if state == PreambleState::Return {
                return Preamble {
                    pointer_struct_name: pointer_struct_name.unwrap(),
                    loader_function_name: loader_function_name.unwrap(),
                }
            }

            let token = tokens.next();
            if token.is_none() {
                panic!("Ran out of tokens while procesing preamble");
            }

            let token = token.unwrap();
            match state {
                PreambleState::StructLiteral => {
                    let mut ok = false;
                    match &token {
                        TokenTree::Ident(ident) => {
                            if ident.to_string() == "struct" {
                                ok = true
                            }
                        },
                        _ => { }
                    }

                    if !ok {
                        panic!("Expected literal 'struct' in preamble, got {:?}", token);
                    }
                    state = PreambleState::StructName;
                },

                PreambleState::StructName => {
                    match &token {
                        TokenTree::Ident(ident) => {
                            pointer_struct_name = Some(ident.clone());
                        },
                        _ => panic!("expected identifier")
                    }

                    state = PreambleState::LoaderLiteral;
                },

                PreambleState::LoaderLiteral => {
                    let mut ok = false;
                    match &token {
                        TokenTree::Ident(ident) => {
                            if ident.to_string() == "loader" {
                                ok = true
                            }
                        },
                        _ => { }
                    }

                    if !ok {
                        panic!("Expected literal 'loader' in preamble, got {:?}", token);
                    }
                    state = PreambleState::LoaderName
                },

                PreambleState::LoaderName => {
                    match &token {
                        TokenTree::Ident(ident) => {
                            loader_function_name = Some(ident.clone());
                        },
                        _ => panic!("expected identifier")
                    }

                    state = PreambleState::Return;
                },

                PreambleState::Return => unreachable!(),
            }
        }
    }
}

struct FunctionDefinitions {
    functions: Punctuated<FunctionDefinition, Token![;]>
}

impl Parse for FunctionDefinitions {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(FunctionDefinitions {
            functions: Punctuated::parse_terminated(input)?,
        })
    }
}

struct FunctionDefinition {
    _fn_token: Token![fn],
    fn_name: Ident,                // why is this one not from token::?
    _paren_token: token::Paren,
    args: Punctuated<FnArg, Token![,]>,
    _arrow: Token![->],
    ret: syn::Type,
}

impl Parse for FunctionDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(FunctionDefinition {
            _fn_token: input.parse()?,
            fn_name: input.parse()?,
            _paren_token: parenthesized!(content in input),
            args: content.parse_terminated(FnArg::parse)?,
            _arrow: input.parse()?,
            ret: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn vk_functions(strm: TokenStream) -> TokenStream {
    let mut iter = strm.into_iter();
    let preamble = Preamble::new(&mut iter);

    let mut remain = TokenStream::new();
    remain.extend(iter);

    let functions = parse_macro_input!(remain as FunctionDefinitions);

    let mut pointers = Vec::new();
    for function in &functions.functions {
        let nm = &function.fn_name;

        let mut args = Vec::new(); // hack
        for arg in &function.args {
            args.push( arg.clone() );
        }

        let ret = &function.ret;
        pointers.push(quote! {
            pub #nm : unsafe extern "C" fn( #(#args),* ) -> #ret
        });
    }

    let mut loaders = Vec::new();
    for function in &functions.functions {
        let nm = &function.fn_name;
        loaders.push(quote! {
            #nm: std::mem::transmute(
                check_non_null(
                    stringify!( #nm ),
                    resolver( arg,
                              concat!( stringify!( #nm ), "\0" ).as_bytes().as_ptr() as *const std::os::raw::c_char ) ) )
            
        });
    }

    let rname = syn::Ident::new(&preamble.loader_function_name.to_string(), preamble.loader_function_name.span().into());
    let sname = syn::Ident::new(&preamble.pointer_struct_name.to_string(), preamble.pointer_struct_name.span().into());
    let tokens = quote! {
        pub struct #sname {
            #(#pointers),*
        }

        impl #sname {
            pub fn load<F>(mut f: F) -> Self
                where F: FnMut( &std::ffi::CStr ) -> *const std::os::raw::c_void
            {
                Self::load_with_arg( std::ptr::null(), f )
            }

            pub fn load_with_arg<F>(arg: *const std::os::raw::c_void, mut f: F) -> Self
                where F: FnMut( &std::ffi::CStr ) -> *const std::os::raw::c_void
            {
                type Resolver =  unsafe extern "C" fn(
                    *const std::os::raw::c_void,
                    *const std::os::raw::c_char) -> *const std::os::raw::c_void;

                fn check_non_null(n: &str, s: *const c_void) -> *const c_void {
                    if s.is_null() {
                        panic!("Resolved symbol {} was NULL", n);
                    }
                    return s;
                }

                unsafe {
                    let nm = std::ffi::CStr::from_ptr(
                        concat!(stringify!(#rname), "\0").as_bytes().as_ptr() as *const std::os::raw::c_char );
                    let resolver: Resolver = std::mem::transmute( f( nm ) );

                    Self {
                        #(#loaders),*
                    }
                }
            }
        }
    };

    proc_macro::TokenStream::from(tokens)
}
