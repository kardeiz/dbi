use proc_macro::TokenStream;
use proc_macro2::Span;

use syn::*;
use quote::quote;

pub fn dbi_trait(attrs: TokenStream, item: TokenStream) -> TokenStream {

    let attrs = parse_macro_input!(attrs as AttributeArgs);
    let mut item = parse_macro_input!(item as ItemTrait);

    let item_ident = &item.ident;

    let impl_ident = attrs.iter()
        .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
        .filter_map(|x| match x { Meta::List(y) => Some(y), _ => None })
        .filter(|x| x.ident == "impl_for" )
        .flat_map(|x| x.nested.iter() )
        .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
        .filter_map(|x| match x { Meta::NameValue(y) => Some(y), _ => None })
        .find(|x| x.ident == "new" )
        .and_then(|x| match x.lit { Lit::Str(ref y) => Some(y), _ => None })
        .and_then(|x| x.parse::<Ident>().ok() )
        .unwrap();

    let impl_quote = quote! {
        pub struct #impl_ident<T>(pub T);
    };

    let impl_conn = quote! {
        impl<F, I> _dbi::Connection for #impl_ident<F> 
            where F: _dbi::exp::futures::Future<Item=I, Error=_dbi::exp::my::errors::Error> + Send + 'static, 
                I: _dbi::exp::my::prelude::Queryable {
            type Queryable = I;
            type Inner = F;
            fn connection(self) -> Self::Inner {
                self.0
            }
        }
    };

    let trait_item_methods = item.items.clone().into_iter()
        .filter_map(|x| match x { TraitItem::Method(y) => Some(y), _ => None })
        .collect::<Vec<_>>();

    let mut impl_methods = Vec::new();

    for trait_item_method in trait_item_methods {
        
        // panic!("{:#?}", trait_item_method.attrs);
        
        let main_list_attr = match trait_item_method.attrs.iter()
            .flat_map(|x| x.parse_meta() )
            .filter_map(|x| match x { Meta::List(y) => Some(y), _ => None })
            .next() {
            Some(x) => x,
            _ => { continue; } 
        };

        if main_list_attr.ident == "sql_query" {

            let sql_query_attr = &main_list_attr;

            let sql = sql_query_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Literal(y) => Some(y), _ => None })
                .next()
                .unwrap();

            let mut mapper = quote!(_dbi::exp::my::prelude::FromRow::from_row_opt);

            if let Some(expr) = sql_query_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
                .filter_map(|x| match x { Meta::NameValue(y) => Some(y), _ => None })
                .find(|x| x.ident == "mapper" )
                .and_then(|x| match x.lit { Lit::Str(ref y) => Some(y), _ => None })
                .map(|x| x.parse::<Expr>().unwrap() ) {     
                mapper = quote!(#expr);        
            }

            let use_named_params = sql_query_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
                .filter_map(|x| match x { Meta::NameValue(y) => Some(y), _ => None })
                .find(|x| x.ident == "use_named_params" )
                .and_then(|x| match x.lit { Lit::Bool(ref y) => Some(y), _ => None })
                .map(|x| x.value )
                .unwrap_or(false);

            let MethodSig { constness, unsafety, asyncness, abi, ident, decl } = trait_item_method.sig.clone();

            let params = decl.inputs.iter()
                .filter_map(|x| match *x { FnArg::Captured(ref y) => Some(y), _ => None })
                .filter_map(|x| match x.pat { Pat::Ident(ref y) => Some(y), _ => None })
                .map(|x| &x.ident )
                .collect::<Vec<_>>();

            let params = if params.is_empty() {
                quote!(().into())
            } else {
                if use_named_params {
                    let tups = params.iter().map(|x| {
                        let s = x.to_string();
                        quote!((#s, _dbi::exp::my::Value::from(#x)))
                    }).collect::<Vec<_>>();
                    quote!(_dbi::exp::my::Params::from(vec![#(#tups),*]))
                } else {
                    let params = params.iter().map(|x| 
                        quote!(&#x as &_dbi::exp::my::prelude::ToValue)
                    ).collect::<Vec<_>>();

                    quote!(_dbi::exp::my::Params::from([#(#params),*].as_ref()))
                }            
            };

            let new_fn: ItemFn = parse_quote!{
                fn __() {
                    let rt = _dbi::utils::query(self.connection(), #sql, #params, #mapper)
                        .map(|x| x.into() );
                    Box::new(rt)
                }
            };

            let new_fn = ItemFn { 
                constness, 
                unsafety, 
                asyncness, 
                abi, 
                ident, 
                decl: Box::new(decl.clone()), 
                attrs: vec![],
                ..new_fn 
            };

            impl_methods.push(quote!(#new_fn));
        }

        if main_list_attr.ident == "sql_update" {
            let sql_update_attr = &main_list_attr;

            let sql = sql_update_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Literal(y) => Some(y), _ => None })
                .next()
                .unwrap();

            let use_named_params = sql_update_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
                .filter_map(|x| match x { Meta::NameValue(y) => Some(y), _ => None })
                .find(|x| x.ident == "use_named_params" )
                .and_then(|x| match x.lit { Lit::Bool(ref y) => Some(y), _ => None })
                .map(|x| x.value )
                .unwrap_or(false);

            let get_last_insert_id = sql_update_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
                .filter_map(|x| match x { Meta::NameValue(y) => Some(y), _ => None })
                .find(|x| x.ident == "get_last_insert_id" )
                .and_then(|x| match x.lit { Lit::Bool(ref y) => Some(y), _ => None })
                .map(|x| x.value )
                .unwrap_or(false);

            let MethodSig { constness, unsafety, asyncness, abi, ident, decl } = trait_item_method.sig.clone();

            let params = decl.inputs.iter()
                .filter_map(|x| match *x { FnArg::Captured(ref y) => Some(y), _ => None })
                .filter_map(|x| match x.pat { Pat::Ident(ref y) => Some(y), _ => None })
                .map(|x| &x.ident )
                .collect::<Vec<_>>();

            let params = if params.is_empty() {
                quote!(().into())
            } else {
                if use_named_params {
                    let tups = params.iter().map(|x| {
                        let s = x.to_string();
                        quote!((#s, _dbi::exp::my::Value::from(#x)))
                    }).collect::<Vec<_>>();
                    quote!(_dbi::exp::my::Params::from(vec![#(#tups),*]))
                } else {
                    let params = params.iter().map(|x| 
                        quote!(&#x as &_dbi::exp::my::prelude::ToValue)
                    ).collect::<Vec<_>>();

                    quote!(_dbi::exp::my::Params::from([#(#params),*].as_ref()))
                }            
            };

            // panic!("{}", params);

            let new_fn: ItemFn = parse_quote!{
                fn __() {
                    let rt = _dbi::utils::update(self.connection(), #sql, #params, #get_last_insert_id)
                        .map(|x| x.into() );
                    Box::new(rt)
                }
            };

            let new_fn = ItemFn { 
                constness, 
                unsafety, 
                asyncness, 
                abi, 
                ident, 
                decl: Box::new(decl.clone()), 
                attrs: vec![],
                ..new_fn 
            };

            impl_methods.push(quote!(#new_fn));
        }

        if main_list_attr.ident == "sql_batch" {
            let sql_batch_attr = &main_list_attr;

            let sql = sql_batch_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Literal(y) => Some(y), _ => None })
                .next()
                .unwrap();

            let use_named_params = sql_batch_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
                .filter_map(|x| match x { Meta::NameValue(y) => Some(y), _ => None })
                .find(|x| x.ident == "use_named_params" )
                .and_then(|x| match x.lit { Lit::Bool(ref y) => Some(y), _ => None })
                .map(|x| x.value )
                .unwrap_or(false);

            let get_last_insert_id = sql_batch_attr.nested.iter()
                .filter_map(|x| match x { NestedMeta::Meta(y) => Some(y), _ => None })
                .filter_map(|x| match x { Meta::NameValue(y) => Some(y), _ => None })
                .find(|x| x.ident == "get_last_insert_id" )
                .and_then(|x| match x.lit { Lit::Bool(ref y) => Some(y), _ => None })
                .map(|x| x.value )
                .unwrap_or(false);

            let MethodSig { constness, unsafety, asyncness, abi, ident, decl } = trait_item_method.sig.clone();

            let params = decl.inputs.iter()
                .filter_map(|x| match *x { FnArg::Captured(ref y) => Some(y), _ => None })
                .filter_map(|x| match x.pat { Pat::Ident(ref y) => Some(y), _ => None })
                .map(|x| &x.ident )
                .collect::<Vec<_>>();

            let params = if params.is_empty() {
                quote!(().into())
            } else {
                if use_named_params {                    
                    let pat = {
                        let mut params_iter = params.iter();
                        let first = params_iter.next();
                        params_iter.fold(quote!(#first), |acc, p| {
                            quote!((#acc, #p))
                        })
                    };

                    let iter = {
                        let mut params_iter = params.iter();
                        let first = params_iter.next();
                        params_iter.fold(quote!(#first.into_iter()), |acc, p| {
                            quote!(#acc.zip(#p.into_iter()))
                        })
                    };

                    let tups = params.iter().map(|x| {
                        let s = x.to_string();
                        quote!((#s, _dbi::exp::my::Value::from(#x)))
                    }).collect::<Vec<_>>();

                    quote! {
                        #iter.map(|#pat| _dbi::exp::my::Params::from(vec![#(#tups),*]) )
                            .collect::<Vec<_>>()
                    }
                } else {
                    let pat = {
                        let mut params_iter = params.iter();
                        let first = params_iter.next();
                        params_iter.fold(quote!(#first), |acc, p| {
                            quote!((#acc, #p))
                        })
                    };

                    let iter = {
                        let mut params_iter = params.iter();
                        let first = params_iter.next();
                        params_iter.fold(quote!(#first.into_iter()), |acc, p| {
                            quote!(#acc.zip(#p.into_iter()))
                        })
                    };

                    let params = params.iter().map(|x| 
                        quote!(&#x as &_dbi::exp::my::prelude::ToValue)
                    ).collect::<Vec<_>>();

                    quote! {
                        #iter.map(|#pat| _dbi::exp::my::Params::from([#(#params),*].as_ref()) )
                            .collect::<Vec<_>>()
                    }
                }            
            };

            let new_fn: ItemFn = parse_quote!{
                fn __() {
                    let rt = _dbi::utils::batch(self.connection(), #sql, #params)
                        .map(|x| x.into() );
                    Box::new(rt)
                }
            };

            let new_fn = ItemFn { 
                constness, 
                unsafety, 
                asyncness, 
                abi, 
                ident, 
                decl: Box::new(decl.clone()), 
                attrs: vec![],
                ..new_fn 
            };

            impl_methods.push(quote!(#new_fn));
        }


    }

    for meth in item.items.iter_mut()
        .filter_map(|x| match x { TraitItem::Method(y) => Some(y), _ => None }) {
        meth.attrs = vec![];        
    }
        

    let dummy_const = Ident::new(
        &format!("_IMPL_{}_FOR_{}", item_ident.to_string(), impl_ident.to_string()),
        Span::call_site()
    );

    let out = quote!{
        #item
        #impl_quote
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #[allow(unknown_lints)]
            #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
            extern crate dbi as _dbi;
            #impl_conn
            impl<T> #item_ident for #impl_ident<T> where Self: _dbi::Connection {
                #(#impl_methods)*
            }
        };
    };

    // panic!("{}", out);

    out.into()
}

pub fn from_row_macro_derive(item: TokenStream) -> TokenStream {
    
    let mut item = parse_macro_input!(item as DeriveInput);

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

                fields.push(quote! {
                    let #ident1: #ty = match row.get_opt(#ident2) {
                        Some(Ok(val)) => val,
                        _ => { return Err(_dbi::exp::my::FromRowError(row)); }
                    };
                });

                field_names.push(ident1);
            }
        }
    }

    let item_ident1 = item.ident.clone();
    let item_ident2 = item_ident1.clone();

    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let impl_inner = quote! { 
        impl #impl_generics _dbi::exp::my::prelude::FromRow for #item_ident1 #ty_generics #where_clause {
            fn from_row(row: _dbi::exp::my::Row) -> Self {                
                Self::from_row_opt(row).unwrap()
            }
            fn from_row_opt(row: _dbi::exp::my::Row) -> Result<Self, _dbi::exp::my::FromRowError> {
                #(#fields)*
                Ok(#item_ident2 {
                    #(#field_names),*
                })
            }
        }
    };

    let dummy_const = Ident::new(
        &format!("_IMPL_FromRow_FOR_{}", item.ident.to_string()),
        Span::call_site()
    );

    let out = quote!{
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const #dummy_const: () = {
            #[allow(unknown_lints)]
            #[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
            extern crate dbi as _dbi;
            #impl_inner
        };
    };

    out.into()
}