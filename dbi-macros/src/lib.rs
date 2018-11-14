#![recursion_limit="4096"]

extern crate proc_macro;
extern crate quote;
extern crate syn;
extern crate proc_macro2;

use proc_macro::TokenStream;

#[cfg(feature="mysql")]
mod mysql;

#[cfg(feature="mysql")]
#[proc_macro_attribute]
pub fn dbi_trait(attrs: TokenStream, item: TokenStream) -> TokenStream {
    mysql::dbi_trait(attrs, item)
}

#[cfg(feature="mysql")]
#[proc_macro_derive(FromRow, attributes(dbi))]
pub fn from_row_macro_derive(item: TokenStream) -> TokenStream {
    mysql::from_row_macro_derive(item)
}
