#![recursion_limit="4096"]

extern crate proc_macro;
extern crate quote;
extern crate syn;

extern crate futures;
extern crate mysql_async as my;

use proc_macro::TokenStream;
use quote::quote;

use syn::{
    parse_macro_input, 
    parse_quote,
    DeriveInput, 
    Expr,
    Ident,
    Item, 
    ItemFn, 
    FnArg,
    AttributeArgs,
    Lit,
    LitStr,
    NestedMeta,
    Meta,
    MethodSig,
    Stmt,
    TraitItemMethod,
    Token
};

use futures::Future;
use quote::ToTokens;



#[proc_macro_attribute]
pub fn sql_query(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attrs as AttributeArgs);
    let mut item = parse_macro_input!(item as TraitItemMethod);
    
    let mut query = None;
    let mut mapper = None;

    for meta in &attrs {
        if let NestedMeta::Literal(ref lit) = meta {
            query = Some(lit);
        } else if let NestedMeta::Meta(Meta::NameValue(ref meta_name_value)) = meta {
            if &meta_name_value.ident == "mapper" {
                if let Lit::Str(ref ls) = meta_name_value.lit {
                    if let Ok(ty) = ls.parse::<Expr>() {
                        mapper = Some(ty);
                    }                    
                }
            }
        }
    }


    let query = query.unwrap();
    let mapper = mapper.unwrap();

    let MethodSig { constness, unsafety, asyncness, abi, ident, decl } = item.sig;

    let params = decl.inputs.iter()
        .filter_map(|p| if let FnArg::Captured(ref q) = *p { Some(q) } else { None } )
        .map(|p| p.pat.clone() )
        .collect::<Vec<_>>();

    let params = if params.is_empty() {
        quote!(())
    } else {
        quote!((#(#params,)*))
    };

    let mut new_fn: ItemFn = parse_quote!{
        fn __() {
            use my::prelude::Queryable;

            trait FirstOrAll {
                type Item;
                fn first_or_all(items: Vec<Self::Item>) -> Self;
            }

            impl<T> FirstOrAll for Vec<T> {
                type Item = T;
                fn first_or_all(items: Vec<Self::Item>) -> Self {
                    items
                }
            }

            impl<T> FirstOrAll for Option<T> {
                type Item = T;
                fn first_or_all(items: Vec<Self::Item>) -> Self {
                    items.into_iter().next()
                }
            }

            let sql = #query;
            let rt = self.connection().and_then(move |conn| {
                conn.prep_exec(sql, #params)
            }).and_then(move |res| {
                res.map_and_drop(|row| {
                    let tup = my::from_row(row);
                    #mapper(tup)
                })
            }).map(|(_, val)| {
                FirstOrAll::first_or_all(val)
            });

            Box::new(rt)
        }
    };

    new_fn.constness = constness;
    new_fn.unsafety = unsafety;
    new_fn.asyncness = asyncness;
    new_fn.abi = abi;
    new_fn.ident = ident;
    new_fn.decl = Box::new(decl);

    let out = quote!(#new_fn);

    // panic!("{}", &out);

    out.into()

}