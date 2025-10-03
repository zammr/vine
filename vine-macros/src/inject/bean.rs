use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{FnArg, GenericArgument, ItemFn, LitStr, parse_macro_input, parse_quote, PathArguments, PatType, ReturnType, Signature, Type};
use crate::inject::bean_field::BeanField;
use crate::inject::generate_value_based_on_config;

pub fn generate_setup_fn_for_bean(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let ItemFn {
        vis,
        sig: Signature { output, ident, inputs, .. },
        block,
        ..
    } = parse_macro_input!(input as ItemFn);

    let args: Vec<_> = inputs.iter().map(|arg| {
        let FnArg::Typed(PatType { pat, ty, .. }) = arg else { panic!("unsupported FnArg"); };
        return quote!(#pat: #ty)
    }).collect();

    let resolved_fn_args: Vec<_> = inputs.iter().map(|fn_arg| match parse_quote!(#fn_arg) {
        BeanField::Bean(_, ty, name) => quote!(ctx.get_bean::<#ty>(#name)?,),
        BeanField::PrimaryBean(_, ty) => quote!(ctx.get_primary_bean::<#ty>()?,),
        BeanField::Beans(_, ty) => quote!(ctx.get_beans::<#ty>()?,),
        BeanField::Value(_, ty, value) => {
            let compute_call = generate_value_based_on_config(&ty, &value);
            quote!({
                let config = ctx.get_bean::<dyn vine::vine_core::config::PropertyResolver + Send + Sync>("config")?;
                #compute_call
            },)
        },
    }).collect();

    let fn_name_str = ident.to_string();
    let bean_name = LitStr::new(&fn_name_str, Span::call_site());

    let setup_ident = format!("SETUP_{}", fn_name_str.to_uppercase());
    let setup_ident = Ident::new(&setup_ident, Span::call_site());

    let ty = get_create_fn_output(&output);
    let extended = quote!(
        #[vine::distributed_slice(vine::vine_core::context::auto_register_context::SETUP)]
        pub static #setup_ident: fn(&vine::vine_core::context::context::Context) -> Result<(), vine::vine_core::core::Error> = |ctx| {
            let ty = vine::vine_core::core::ty::Type::of::<#ty>();
            ty.add_downcast::<#ty>(|b| Ok(std::sync::Arc::downcast::<#ty>(b)?));

            let bean_def = vine::vine_core::core::bean_def::BeanDef::builder()
                .name(#bean_name)
                .ty(ty)
                .get(std::sync::Arc::new(|ctx| Ok(#ident(#(#resolved_fn_args)*))))
                .build();
            ctx.register(bean_def)
        };

        #vis fn #ident (#(#args),*) #output { #block }
    );

    extended.into()
}

fn get_create_fn_output(return_type: &ReturnType) -> Type {
    let ReturnType::Type(_, box_type) = return_type else { panic!() };
    let Type::Path(path_type) =  box_type.as_ref() else { panic!() };
    let Some(path_seg) = path_type.path.segments.first() else { panic!() };

    let PathArguments::AngleBracketed(x) = &path_seg.arguments else { panic!() };
    let Some(GenericArgument::Type(class)) = x.args.first() else { panic!() };

    class.clone()
}