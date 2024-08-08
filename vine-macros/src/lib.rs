use proc_macro::TokenStream;

use crate::inject::bean::generate_setup_fn_for_bean;
use crate::inject::bean_derive::generate_setup_fn_for_bean_derive;
use crate::inject::injectable::generate_setup_fn_for_injectable;

mod inject;

/// TODO: add documentation
#[proc_macro_attribute]
pub fn bean(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_setup_fn_for_bean(attr, item)
}

/// TODO: add documentation
#[proc_macro_derive(Bean, attributes(value, qualifier))]
pub fn bean_derive(input: TokenStream) -> TokenStream {
    generate_setup_fn_for_bean_derive(input)
}

/// TODO: add documentation
#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate_setup_fn_for_injectable(attr, item)
}