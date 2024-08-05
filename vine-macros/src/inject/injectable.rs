use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, ItemImpl, parse_macro_input};

pub fn generate_setup_fn_for_injectable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_impl = parse_macro_input!(item as ItemImpl);
    let Some((_, trait_ident, _)) = &item_impl.trait_ else { panic!("cannot resolve trait") };
    let ty = item_impl.self_ty.as_ref();

    let setup_fn_name = format!("SETUP_{}_ALIAS_OF_{}", quote!(#trait_ident).to_string().to_uppercase(), quote!(#ty).to_string().to_uppercase());
    let setup_fn = Ident::new(&setup_fn_name, Span::call_site());

    let extended = quote!(
        #[linkme::distributed_slice(vine::vine_core::context::SETUP)]
        pub static #setup_fn: fn(&vine::vine_core::Context) -> Result<(), vine::vine_core::Error> = |_| {
            let ty = vine::vine_core::core::r#type::Type::of::<#ty>();
            ty.add_downcast::<dyn #trait_ident + Send + Sync>(|b| Ok(std::sync::Arc::downcast::<#ty>(b)?));
            Ok(())
        };

        #item_impl
    );

    extended.into()
}