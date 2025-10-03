use quote::quote;
use syn::{LitStr, Type};

pub mod bean;
pub mod injectable;
pub mod bean_derive;
pub mod bean_field;

fn generate_value_based_on_config(ty: &Type, value: &LitStr) -> proc_macro2::TokenStream {
    let Type::Path(type_path) = ty else {
        return quote!(config.compute_template_value(#value)?);
    };

    let Some(segment) = type_path.path.segments.last() else {
        return quote!(config.compute_template_value(#value)?);
    };

    let type_name = segment.ident.to_string();

    match type_name.as_str() {
        "String" => quote!(config.compute_template_value(#value)?),
        "bool" => quote!(config.compute_template_value_as_bool(#value)?),
        "i8" => quote!(config.compute_template_value_as_i8(#value)?),
        "i16" => quote!(config.compute_template_value_as_i16(#value)?),
        "i32" => quote!(config.compute_template_value_as_i32(#value)?),
        "i64" => quote!(config.compute_template_value_as_i64(#value)?),
        "u8" => quote!(config.compute_template_value_as_u8(#value)?),
        "u16" => quote!(config.compute_template_value_as_u16(#value)?),
        "u32" => quote!(config.compute_template_value_as_u32(#value)?),
        "u64" => quote!(config.compute_template_value_as_u64(#value)?),
        "f32" => quote!(config.compute_template_value_as_f32(#value)?),
        "f64" => quote!(config.compute_template_value_as_f64(#value)?),
        _ => quote!(config.compute_template_value(#value)?),
    }
}