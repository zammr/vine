use syn::{AngleBracketedGenericArguments, Field, GenericArgument, Ident, LitStr, PathArguments, PathSegment, Result, Type};
use syn::parse::{Parse, ParseStream};

pub enum BeanField {
    Bean(Ident, Type, LitStr),
    PrimaryBean(Ident, Type),
    Beans(Ident, Type),
    Value(Ident, Type, LitStr)
}

fn get_inner_type(ty: &Type) -> (&Ident, &Type) {
    let Type::Path(type_path) = ty else { panic!("?????") };
    let Some(PathSegment { ident, arguments: PathArguments::AngleBracketed( AngleBracketedGenericArguments { args, .. }) }) = type_path.path.segments.first() else { panic!("?????") };
    let Some(GenericArgument::Type(inner_type)) = args.first() else { panic!("TODO") };

    (ident, inner_type)
}

impl Parse for BeanField {
    fn parse(input: ParseStream) -> Result<Self> {
        let Field {
            ident: Some(ident), attrs, ty,
        .. } = input.call(Field::parse_named)? else { panic!("failed to parse BeanField") };

        if let Some(attr) = attrs.first() {
            if attr.path().is_ident("qualifier") {
                let bean_name = attr.parse_args::<LitStr>()?;
                let (_, ty) = get_inner_type(&ty);
                return Ok(BeanField::Bean(ident, ty.clone(), bean_name));
            } else if attr.path().is_ident("value") {
                let value_template = attr.parse_args::<LitStr>()?;
                return Ok(BeanField::Value(ident, ty, value_template));
            }
        }

        let (ty_ident, ty) = get_inner_type(&ty);
        if ty_ident.eq("Vec") {
            let (_, ty) = get_inner_type(ty);
            return Ok(BeanField::Beans(ident, ty.clone()));
        } else if ty_ident.eq("Arc") {
            return Ok(BeanField::PrimaryBean(ident, ty.clone()));
        }

        todo!()
    }
}
