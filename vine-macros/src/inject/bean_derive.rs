use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Ident, LitStr, parse_macro_input, parse_quote};

use crate::inject::bean_field::BeanField;

pub fn generate_setup_fn_for_bean_derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data: Data::Struct(DataStruct { fields, .. }),
        ..
    } = parse_macro_input!(input) else { panic!("") };

    let resolved_fields: Vec<_> = fields.iter().map(|field| match parse_quote!(#field) {
        BeanField::Bean(field, ty, name) => quote!(#field: ctx.get_bean::<#ty>(#name)?,),
        BeanField::PrimaryBean(field, ty) => quote!(#field: ctx.get_primary_bean::<#ty>()?,),
        BeanField::Beans(field, ty) => quote!(#field: ctx.get_beans::<#ty>()?,),
        BeanField::Value(field, _ty, value) => quote!(#field: {
            let config = ctx.get_bean::<dyn vine::vine_core::config::PropertyResolver + Send + Sync>("config")?;
            config.compute_template_value(#value)?
        },),
    }).collect();

    let setup_ident = format!("SETUP_{}", &ident.to_string().to_uppercase());
    let setup_ident = Ident::new(&setup_ident, Span::call_site());

    let bean_name = &ident.to_string();
    let bean_name = LitStr::new(&bean_name, Span::call_site());

    let output = quote! {
        #[vine::distributed_slice(vine::vine_core::context::auto_register_context::SETUP)]
        pub static #setup_ident: fn(&vine::vine_core::context::context::Context) -> Result<(), vine::vine_core::core::Error> = |ctx| {
            let ty = vine::vine_core::core::ty::Type::of::<#ident>();
            ty.add_downcast::<#ident>(std::sync::Arc::downcast::<#ident>);
            let bean_def = vine::vine_core::core::bean_def::BeanDef::builder()
                .name(#bean_name)
                .ty(ty)
                .get(std::sync::Arc::new(|ctx| Ok(std::sync::Arc::new(#ident {
                    #(#resolved_fields)*
                }))))
                .build();
            ctx.register(bean_def)
        };
    };
    output.into()
}