use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, LitStr, Signature};

pub fn generate_init_fn_for_controller(item_impl: ItemImpl) -> TokenStream {
    let vine_setup = quote!(vine::vine_core::context::auto_register_context::SETUP);
    let vine_context = quote!(vine::vine_core::context::context::Context);
    let vine_error = quote!(vine::vine_core::core::Error);
    let vine_web = quote!(vine::vine_axum::Web);

    let ItemImpl { self_ty, items, .. } = &item_impl;

    let routes: Vec<_>  = items.iter()
        .flat_map(|impl_item| handler_to_method_router(impl_item))
        .map(|route| quote!(web.add_route(#route);) )
        .collect();

    let ty_name = quote!(#self_ty).to_string();
    let controller = LitStr::new(ty_name.as_str(), Span::call_site());
    let setup_ident = Ident::new(format!("SETUP_INIT_FN_{}_CONTROLLER", ty_name.to_uppercase()).as_str(), Span::call_site());

    quote!(
        #[vine::distributed_slice(#vine_setup)]
        pub static #setup_ident: fn(&#vine_context) -> Result<(), #vine_error> = |ctx| {
            ctx.add_init_fn(#controller, std::sync::Arc::new(|ctx| {
                let web = ctx.get_primary_bean::<#vine_web>()?;
                let controller = ctx.get_primary_bean::<#self_ty>()?;

                #(#routes)*

                Ok(())
            }))
        };

        #item_impl
    )
}

fn handler_to_method_router(item: &ImplItem) -> Vec<proc_macro2::TokenStream> {
    let ImplItem::Fn(ImplItemFn {
        attrs,
        sig: Signature { ident, inputs, .. },
    .. }) = item else { panic!("TODO: better explanation" )};

    let mut args = quote!();
    for (arg_index, arg) in inputs.iter().enumerate() {
        if let FnArg::Typed(_) = arg {
            let arg_ident = Ident::new(format!("arg{}", arg_index).as_str(), Span::call_site());
            args = quote!(#args #arg_ident,)
        }
    }

    let handler = quote!({
        // TODO: check async/sync
        let controller = std::sync::Arc::clone(&controller);
        move |#args| async move { controller.#ident(#args).await }
    });

    let mut routes = vec![];
    for attr in attrs {
        let http_path = attr.parse_args::<LitStr>().unwrap();
        let htpp_method = match attr.path() {
            path if path.is_ident("patch") || path.is_ident("vine::patch") => quote!(axum::routing::patch),
            path if path.is_ident("options") || path.is_ident("vine::options") => quote!(axum::routing::options), 
            path if path.is_ident("trace") || path.is_ident("vine::trace") => quote!(axum::routing::trace),
            path if path.is_ident("connect") || path.is_ident("vine::connect") => quote!(axum::routing::connect),
            path if path.is_ident("get") || path.is_ident("vine::get") => quote!(axum::routing::get),
            path if path.is_ident("head") || path.is_ident("vine::head") => quote!(axum::routing::head),
            path if path.is_ident("post") || path.is_ident("vine::post") => quote!(axum::routing::post),
            path if path.is_ident("put") || path.is_ident("vine::put") => quote!(axum::routing::put),
            path if path.is_ident("delete") || path.is_ident("vine::delete") => quote!(axum::routing::delete),
            _ => continue
        };
        routes.push(quote!(
            #http_path.to_string(), #htpp_method(#handler)
        ));
    }

    routes
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, ItemImpl};

    #[test]
    fn test_to_routes() {
        let item_impl: ItemImpl = parse_quote! {
            impl TestController {
                #[get("/test")]
                async fn test_get(&self) -> &'static str {
                    "test"
                }

                #[post("/create")]
                async fn test_post(&self, body: String) -> String {
                    body
                }
            }
        };

        let token_stream = generate_init_fn_for_controller(item_impl);
        let syntax_tree: syn::File = parse_quote! {
            #token_stream
        };
        let unparse = prettyplease::unparse(&syntax_tree);
       
        println!("{}", &unparse);
    }
}