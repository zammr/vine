mod controller;

use proc_macro::TokenStream;
use syn::{Item, parse_macro_input};
use crate::controller::generate_init_fn_for_controller;

/// Generates a controller implementation with initialization functions for web routing.
/// 
/// This macro should be applied to an impl block. It will:
/// - Generate a setup function that registers routes with the web framework
/// - Register the controller with Vine's dependency injection system
/// - Allow HTTP method attributes like `#[get]`, `#[post]` etc. on methods
///
/// # Example
///
/// ```rust
/// #[controller]
/// impl MyController {
///     #[get("/hello")]
///     async fn hello(&self) -> &'static str {
///         "Hello World!"
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    match item {
        Item::Impl(item_impl) => generate_init_fn_for_controller(attr, item_impl),
        _ => panic!("unsupported...")
    }
}

macro_rules! http_method_macro {
    ($name:ident) => {
        #[proc_macro_attribute]
        pub fn $name(_attr: TokenStream, input: TokenStream) -> TokenStream {
            input
        }
    };
}

http_method_macro!(get);
http_method_macro!(head); 
http_method_macro!(post);
http_method_macro!(put);
http_method_macro!(delete);
http_method_macro!(patch);
http_method_macro!(options);
http_method_macro!(trace);
http_method_macro!(connect);
