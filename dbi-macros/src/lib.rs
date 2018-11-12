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
    Data,
    DataStruct,
    DeriveInput, 
    Expr,
    Fields,
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

    let mapper = match mapper {
        Some(mapper) => quote!(#mapper),
        None => quote!(FromRow::from_row)
    };

    let attrs = item.attrs;
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

    let new_fn: ItemFn = parse_quote!{
        fn __() {
            use my::prelude::*;

            let sql = #query;
            let rt = self.connection().and_then(move |conn| {
                conn.prep_exec(sql, #params)
            }).and_then(move |res| {
                res.reduce_and_drop(dbi::ResultSet::None, |mut acc, row| {
                    acc.push((#mapper)(row))
                })
            }).map(|(_, val)| {
                val.into()
            });

            Box::new(rt)
        }
    };
    
    let new_fn = ItemFn { 
        constness, 
        unsafety, 
        asyncness, 
        abi, 
        ident, 
        decl: Box::new(decl), 
        attrs,
        ..new_fn 
    };

    let out = quote!(#new_fn);

    // panic!("{}", &out);

    out.into()

}

#[proc_macro_derive(FromRow, attributes(dbi))]
pub fn from_row_macro_derive(item: TokenStream) -> TokenStream {
    
    let mut item = parse_macro_input!(item as DeriveInput);

    // panic!("{:#?}", &item);

    let mut fields = Vec::new();
    let mut field_names = Vec::new();

    if let Data::Struct(ref ds) = item.data {
        if let Fields::Named(ref nfs) = ds.fields {
            for field in nfs.named.iter() {
                let ident1 = field.ident.clone().unwrap();
                
                let ident2 = ident1.clone().to_string();
                let mut ident2 = quote!(#ident2);

                for attr in &field.attrs {
                    if let Ok(meta) = attr.parse_meta() {                    
                        if let Meta::List(ml) = meta {
                            if ml.ident == "dbi" {
                                for nested in ml.nested.iter() {
                                    if let NestedMeta::Meta(Meta::NameValue(ref mnv)) = nested {
                                        if mnv.ident == "rename" {
                                            let lit = &mnv.lit;
                                            ident2 = quote!(#lit);
                                        }
                                    }
                                }
                            }                        
                        }
                    }
                }

                               
                let ty = &field.ty;

                fields.push(quote!(let #ident1: #ty = row.get(#ident2).unwrap();));

                field_names.push(ident1);
            }
        }
    }

    let item_ident1 = item.ident.clone();
    let item_ident2 = item_ident1.clone();

    let out = quote! { 
        impl my::prelude::FromRow for #item_ident1 {
            fn from_row(row: my::Row) -> Self {                
                #(#fields)*
                #item_ident2 {
                    #(#field_names),*
                }
            }
            fn from_row_opt(row: my::Row) -> Result<Self, my::FromRowError> {
                Ok(Self::from_row(row))
            }
        }
    };

    // panic!("{}", &out);

    out.into()
}