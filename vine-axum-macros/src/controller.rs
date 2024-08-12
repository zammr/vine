use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{ItemImpl, Ident, LitStr, ImplItem, ImplItemFn, Signature, FnArg};

pub fn generate_init_fn_for_controller(_attr: TokenStream, input: ItemImpl) -> TokenStream {
    // TODO: extract to common place
    let vine_setup = quote!(vine::vine_core::context::auto_register_context::SETUP);
    let vine_context = quote!(vine::vine_core::context::context::Context);
    let vine_error = quote!(vine::vine_core::core::Error);
    let vine_web = quote!(vine::vine_axum::Web);

    let ItemImpl { self_ty, items, .. } = &input;


    let routes: Vec<_> = items.iter()
        .flat_map(|impl_item| to_routes(impl_item))
        .map(|route| quote!(web.add_route(#route);) )
        .collect();

    let ty_name = quote!(#self_ty).to_string();
    let controller = LitStr::new(ty_name.as_str(), Span::call_site());
    let setup_ident = Ident::new(format!("SETUP_INIT_FN_{}_CONTROLLER", ty_name).as_str(), Span::call_site());
    let extended = quote!(
        #[linkme::distributed_slice(#vine_setup)]
        pub static #setup_ident: fn(&#vine_context) -> Result<(), #vine_error> = |ctx| {
            ctx.add_init_fn(#controller, std::sync::Arc::new(|ctx| {
                let web = ctx.get_primary_bean::<#vine_web>()?;
                let controller = ctx.get_primary_bean::<#self_ty>()?;

                #(#routes)*

                Ok(())
            }))
        };

        #input
    );

    extended.into()
}

fn to_routes(item: &ImplItem) -> Vec<proc_macro2::TokenStream> {
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
        if attr.path().is_ident("get") || attr.path().is_ident("vine::get") {
            let path = attr.parse_args::<LitStr>().unwrap();
            let route = quote!(
                #path.to_string(), axum::routing::get(#handler)
            );
            routes.push(route);
        } else if attr.path().is_ident("head") || attr.path().is_ident("vine::head") {
            let path = attr.parse_args::<LitStr>().unwrap();
            let route = quote!(
                #path.to_string(), axum::routing::head(#handler)
            );
            routes.push(route);
        } else if attr.path().is_ident("post") || attr.path().is_ident("vine::post") {
            let path = attr.parse_args::<LitStr>().unwrap();
            let route = quote!(
                #path.to_string(), axum::routing::post(#handler)
            );
            routes.push(route);
        } else if attr.path().is_ident("put") || attr.path().is_ident("vine::put") {
            let path = attr.parse_args::<LitStr>().unwrap();
            let route = quote!(
                #path.to_string(), axum::routing::put(#handler)
            );
            routes.push(route);
        } else if attr.path().is_ident("delete") || attr.path().is_ident("vine::delete") {
            let path = attr.parse_args::<LitStr>().unwrap();
            let route = quote!(
                #path.to_string(), axum::routing::delete(#handler)
            );
            routes.push(route);
        }
    }

    routes
}