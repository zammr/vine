mod controller;

use proc_macro::TokenStream;
use syn::{Item, parse_macro_input};
use crate::controller::generate_init_fn_for_controller;

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Impl(item_impl) => generate_init_fn_for_controller(attr, item_impl),
        _ => panic!("unsupported...")
    }
}

#[proc_macro_attribute]
pub fn get(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn head(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn put(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn delete(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}
