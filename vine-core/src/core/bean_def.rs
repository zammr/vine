use std::fmt::{Display, Formatter};
use std::sync::Arc;

use log::trace;

use crate::context::context::Context;
use crate::core::{DynBean, Error};
use crate::core::ty::Type;

pub struct BeanDef {
    name: String,
    ty: Arc<Type>,
    get_fn: Arc<dyn Fn(&Context) -> Result<DynBean, Error> + Send + Sync>,
}

pub struct BeanDefBuilder {
    name: Option<String>,
    ty: Option<Arc<Type>>,
    get_fn: Option<Arc<dyn Fn(&Context) -> Result<DynBean, Error> + Send + Sync>>,
}

impl BeanDef {
    pub fn builder() -> BeanDefBuilder {
        BeanDefBuilder {
            name: None,
            ty: None,
            get_fn: None,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &Arc<Type> {
        &self.ty
    }

    pub fn get(&self, context: &Context) -> Result<(String, DynBean), Error> {
        let name = self.name().to_string();
        trace!("getting Bean(name={}, type={}) with {}", &name, self.ty.name(), context);
        let dyn_bean = self.get_fn.as_ref()(context)?;
        Ok((name, dyn_bean))
    }
}

impl Display for BeanDef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BeanDef(name={}, type={})", &self.name, &self.ty.name())
    }
}


impl BeanDefBuilder {
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn ty(mut self, ty: Arc<Type>) -> Self {
        self.ty = Some(ty);
        self
    }

    pub fn get(mut self, get_fn: Arc<dyn Fn(&Context) -> Result<DynBean, Error> + Send + Sync>) -> Self {
        self.get_fn = Some(get_fn);
        self
    }

    pub fn build(self) -> BeanDef {
        let ty = self.ty.unwrap();
        let get_fn = self.get_fn.unwrap();
        let name = match self.name {
            None => ty.name().to_string(),
            Some(name) => name,
        };

        BeanDef { name, ty, get_fn, }
    }
}