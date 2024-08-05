use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, DeriveInput, DataStruct, parse_macro_input, LitStr, Data, Type, PathArguments, GenericArgument};

pub fn generate_setup_fn_for_bean_derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data: Data::Struct(DataStruct {
            fields,
            .. }),
        ..
    } = parse_macro_input!(input) else {
        panic!("")
    };

    let class = &ident;
    let resolved_fields: Vec<_> = fields.iter().map(|field| {
        let Some(field_name) = &field.ident else { panic!("unsupported field name") };

        let Type::Path(arc_ty) = &field.ty else { panic!("cannot resolve arg Type") };
        let Some(arc_ty) = arc_ty.path.segments.first() else { panic!("cannot resolve arg Type") };
        let PathArguments::AngleBracketed(arc_ty_args) = &arc_ty.arguments else { panic!("cannot resolve arg Type") };
        let Some(GenericArgument::Type(ty)) = arc_ty_args.args.first() else { panic!("cannot resolve arg Type") };

        quote!(#field_name: ctx.get_primary_bean::<#ty>()?,)
    }).collect();


    let setup_ident = format!("SETUP_{}", &ident.to_string().to_uppercase());
    let setup_ident = Ident::new(&setup_ident, Span::call_site());

    let bean_name = &ident.to_string();
    let bean_name = LitStr::new(&bean_name, Span::call_site());

    let output = quote! {
        #[linkme::distributed_slice(vine::vine_core::context::auto_register_context::SETUP)]
        pub static #setup_ident: fn(&vine::vine_core::context::context::Context) -> Result<(), vine::vine_core::core::Error> = |ctx| {
            let ty = vine::vine_core::core::ty::Type::of::<#class>();
            ty.add_downcast::<#class>(|b| Ok(std::sync::Arc::downcast::<#class>(b)?));

            let bean_def = vine::vine_core::core::bean_def::BeanDef::builder()
                .name(#bean_name)
                .ty(ty)
                .get(std::sync::Arc::new(|ctx| Ok(std::sync::Arc::new(#class {
                    #(#resolved_fields)*
                }))))
                .build();
            ctx.register(bean_def)
        };
    };
    output.into()
}