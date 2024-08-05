use proc_macro::TokenStream;

use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{FnArg, GenericArgument, ItemFn, LitStr, parse_macro_input, PathArguments, ReturnType, Type};

pub fn generate_setup_fn_for_bean(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &item_fn.sig.ident;
    let resolved_fn_args: Vec<_> = item_fn.sig.inputs.iter().map(|fn_arg|{
        let FnArg::Typed(type_afn_arg) = fn_arg else { panic!("cannot resolve arg") };
        let Type::Path(arc_ty) = type_afn_arg.ty.as_ref() else { panic!("cannot resolve arg Type") };
        let Some(arc_ty) = arc_ty.path.segments.first() else { panic!("cannot resolve arg Type") };
        let PathArguments::AngleBracketed(arc_ty_args) = &arc_ty.arguments else { panic!("cannot resolve arg Type") };
        let Some(GenericArgument::Type(ty)) = arc_ty_args.args.first() else { panic!("cannot resolve arg Type") };

        quote!(ctx.get_primary_bean::<#ty>()?,)
    }).collect();


    let fn_name_str = fn_name.to_string();
    let setup_ident_name = format!("SETUP_{}", fn_name_str.to_uppercase());
    let setup_ident = Ident::new(&setup_ident_name, Span::call_site());

    // TODO: make many options
    let class = get_create_fn_output(&item_fn.sig.output);
    // let ret = quote!(Ok(b));

    let bean_name = LitStr::new(&fn_name_str, Span::call_site());

    let extended = quote!(
        #[linkme::distributed_slice(vine::vine_core::context::SETUP)]
        pub static #setup_ident: fn(&vine::vine_core::Context) -> Result<(), vine::vine_core::Error> = |ctx| {
            let ty = vine::vine_core::core::r#type::Type::of::<#class>();
            ty.add_downcast::<#class>(|b| Ok(std::sync::Arc::downcast::<#class>(b)?));

            let bean_def = vine::vine_core::core::bean_def::BeanDef::builder()
                .name(#bean_name)
                .ty(ty)
                .get(std::sync::Arc::new(|ctx| Ok(#fn_name(#(#resolved_fn_args)*))))
                .build();
            ctx.register(bean_def)
        };

        #item_fn
    );

    println!("\n\n\n {} \n\n\n", extended);

    extended.into()
}

// enum CreateBeanType {
//     Type(Ident),
//     ResultType(Ident),
//     ArcType(Ident),
//     ResultArcType(Ident),
// }

fn get_create_fn_output(return_type: &ReturnType) -> Type {
    let ReturnType::Type(_, box_type) = return_type else { panic!() };
    let Type::Path(path_type) =  box_type.as_ref() else { panic!() };
    let Some(path_seg) = path_type.path.segments.first() else { panic!() };

    let PathArguments::AngleBracketed(x) = &path_seg.arguments else { panic!() };
    let Some(GenericArgument::Type(class)) = x.args.first() else { panic!() };

    class.clone()
}