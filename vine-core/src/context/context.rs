use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use dashmap::DashMap;
use log::{debug, trace, warn};

use crate::core::{DynBean, Error};
use crate::core::bean_def::BeanDef;
use crate::core::ty::Type;

type InitContextFn = Arc<dyn Fn(&Context) -> Result<(), Error> + Send + Sync>;

#[derive(Clone)]
pub struct Context {
    inner: Arc<InnerContext>,
}

struct InnerContext {
    name: String,
    beans: Arc<DashMap<String, DynBean>>,
    bean_defs: Arc<DashMap<String, Arc<BeanDef>>>,
    contexts: Arc<DashMap<String, Arc<Context>>>,
    init_fns: DashMap<String, InitContextFn>
}

impl Context {
    pub fn new(name: &str) -> Context {
        Context {
            inner: Arc::new(InnerContext {
                name: name.to_string(),
                beans: Default::default(),
                bean_defs: Default::default(),
                contexts: Default::default(),
                init_fns: Default::default(),
            })
        }
    }

    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn add_context(&self, context: Context) {
        // TODO: missed feature (context allow overrides) - if override use warn log. Also think about context property to allow overrides
        self.inner.contexts.insert(context.name().to_string(), Arc::new(context));
    }

    pub fn add_init_fn(&self, name: &str, init_fn: Arc<dyn Fn(&Context) -> Result<(), Error> + Send + Sync>) -> Result<(), Error>{
        // TODO: missed feature (context allow init overrides) - if override use warn log. Also think about context property to allow overrides
        self.inner.init_fns.insert(name.to_string(), init_fn);
        Ok(())
    }

    pub fn init_contexts(&self) -> Result<(), Error> {
        let init_fns = self.inner.get_init_context_fns();

        // TODO: missed feature (disable init function by name)
        for (init_fn_name, init_fn) in init_fns.iter() {
            trace!("execute init fn with name {:?}", init_fn_name);
            init_fn(self)?;
        }

        Ok(())
    }

    pub fn register(&self, bean_def: impl Into<BeanDef>) -> Result<(), Error> {
        let bean_def = bean_def.into();
        if let Some(_) = self.inner.get_bean_def(bean_def.name()) {
            warn!("failed to register duplicated BeanDef(name={}, type={}) in {}", bean_def.name(), bean_def.ty().name(), self);
            return Err(Error::from(format!("failed to register duplicated BeanDef(name={}, type={}) in {}", bean_def.name(), bean_def.ty().name(), self)));
        };

        trace!("registering {} within {}", &bean_def, self);
        self.inner.bean_defs.insert(bean_def.name().to_string(), Arc::new(bean_def));
        Ok(())
    }

    pub fn get_bean<T: ?Sized + 'static>(&self, name: &str) -> Result<Arc<T>, Error> {
        if let Some(dyn_bean) = self.inner.get_bean(name) {
            return Type::downcast::<T>(dyn_bean);
        }

        let Some(bean_def) = self.inner.get_bean_def(name) else {
            warn!("cannot resolve Bean(name={}, type={}) in {}", name, type_name::<T>(), self);
            return Err(Error::from(format!("cannot resolve Bean(name={}, type={}) in {}", name, type_name::<T>(), self)));
        };

        let (name, dyn_bean) = bean_def.get(self)?;
        if let Some(_) = self.inner.beans.insert(name.clone(), dyn_bean.clone()) {
            warn!("unexpected duplicated bean has been created Bean(name={}, type={}) in {}", &name, bean_def.ty().name(), self);
            return Err(Error::from(format!("unexpected duplicated bean has been created Bean(name={}, type={}) in {}", name, bean_def.ty().name(), self)));
        };

        debug!("Bean(name={}, type={}) has been added to {}", &name, bean_def.ty().name(), self);
        return Type::downcast::<T>(dyn_bean);
    }

    pub fn get_primary_bean<T: ?Sized + 'static>(&self) -> Result<Arc<T>, Error> {
        let type_id = TypeId::of::<T>();
        let mut candidates = self.inner.get_bean_defs_by_type(self, &type_id);
        match candidates.len() {
            0 => {
                Err(Error::from(""))
            },
            1 => {
                let bean_def = candidates.pop().unwrap();
                self.get_bean::<T>(bean_def.name())
            },
            _ => {
                todo!("missed feature(primary beans) - add primary to BeanDef and use it to resolve primary bean")
            },
        }
    }

    pub fn get_beans<T: ?Sized + 'static>(&self) -> Result<Vec<Arc<T>>, Error> {
        let type_id = TypeId::of::<T>();

        self.inner.get_bean_defs_by_type(self, &type_id)
            .iter()
            .map(|def| self.get_bean::<T>(def.name()))
            .collect()
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context(name={})", &self.inner.name)
    }
}

impl InnerContext {
    fn get_bean(&self, name: &str) -> Option<DynBean> {
        self.beans.get(name).map(|dyn_ref| dyn_ref.value().clone())
    }
    fn get_bean_def(&self, name: &str) -> Option<Arc<BeanDef>> {
        if let Some(bean_def) = self.bean_defs.get(name) {
            trace!("found {} in Context(name={})", bean_def.value(), &self.name);
            return Some(bean_def.value().clone());
        }

        for ctx in self.contexts.iter() {
            if let Some(bean_def) = ctx.inner.get_bean_def(name) {
                return Some(bean_def);
            }
        }

        trace!("cannot find BeanDef(name={}) in Context(name={})", name, &self.name);
        return None;
    }

    // TODO: missed feature (conditional beans) - use context to check conditional BeanDefs
    fn get_bean_defs_within_context(&self, ctx: &Context) -> Vec<Arc<BeanDef>> {
        let mut bean_defs = Vec::new();
        let mut ctx_defs = self.bean_defs.iter()
            // filter conditional beans here
            .map(|def| def.value().clone())
            .collect();

        bean_defs.append(&mut ctx_defs);
        for child_ctx in self.contexts.iter() {
            let mut ctx_defs = child_ctx.inner.get_bean_defs_within_context(ctx);
            bean_defs.append(&mut ctx_defs);
        }

        return bean_defs;
    }

    fn get_bean_defs_by_type(&self, ctx: &Context, type_id: &TypeId) -> Vec<Arc<BeanDef>> {
        self.get_bean_defs_within_context(ctx).into_iter()
            .filter(|def| def.ty().assignable(type_id))
            .collect()
    }

    fn get_init_context_fns(&self) -> HashMap<String, InitContextFn> {
        let mut fns = HashMap::new();
        for ctx in self.contexts.iter() {
            let mut ctx_fns: Vec<_> = ctx.inner.get_init_context_fns().into_iter().collect();
            while let Some((key, value)) = ctx_fns.pop() {
                fns.insert(key, value);
            }
        }

        let mut ctx_fns: Vec<_> = self.init_fns.iter()
            .map(|item_ref| (item_ref.key().clone(), item_ref.value().clone()))
            .collect();

        while let Some((key, value)) = ctx_fns.pop() {
            fns.insert(key, value);
        }

        return fns;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::context::context::Context;
    use crate::core::bean_def::BeanDef;
    use crate::core::Error;
    use crate::core::ty::Type;

    struct TestBean { name: &'static str, }
    trait TestTrait { fn name(&self) -> &'static str; }
    impl TestTrait for TestBean {
        fn name(&self) -> &'static str { self.name }
    }

    #[test]
    fn register_types_bean_def_then_should_return_and_cast_to_struct_and_dyn_trait() -> Result<(), Error>{
        let ctx = Context::new("test-context");

        let ty = Type::of::<TestBean>();
        ty.add_downcast::<TestBean>(|b| Ok(Arc::downcast::<TestBean>(b)?));
        ty.add_downcast::<dyn TestTrait + Sync + Send>(|b| Ok(Arc::downcast::<TestBean>(b)?));

        let bean_def = BeanDef::builder()
            .ty(ty)
            .name("testBean")
            .get(Arc::new(|_ctx| Ok(Arc::new(TestBean { name: "instance_of_testBean" }))))
            .build();

        ctx.register(bean_def)?;

        let bean = ctx.get_bean::<TestBean>("testBean")?;
        assert_eq!(bean.as_ref().name, "instance_of_testBean");

        let bean = ctx.get_bean::<dyn TestTrait + Sync + Send>("testBean")?;
        assert_eq!(bean.name(), "instance_of_testBean");

        Ok(())
    }

    struct TestBeanWithDep {
        dyn_dep: Arc<dyn TestTrait + Sync + Send>
    }


    #[test]
    fn should_create_and_get_bean_with_dyn_dep() -> Result<(), Error>{
        let ctx = Context::new("test-context");

        let ty = Type::of::<TestBean>();
        ty.add_downcast::<TestBean>(|b| Ok(Arc::downcast::<TestBean>(b)?));
        ty.add_downcast::<dyn TestTrait + Sync + Send>(|b| Ok(Arc::downcast::<TestBean>(b)?));

        let bean_def = BeanDef::builder()
            .ty(ty)
            .name("testBean")
            .get(Arc::new(|_ctx| Ok(Arc::new(TestBean { name: "instance_of_testBean" }))))
            .build();
        ctx.register(bean_def)?;

        let ty = Type::of::<TestBeanWithDep>();
        ty.add_downcast::<TestBeanWithDep>(|b| Ok(Arc::downcast::<TestBeanWithDep>(b)?));

        let bean_def = BeanDef::builder()
            .ty(ty)
            .name("testBeanWithDep")
            .get(Arc::new(|ctx| {
                let bean = Arc::new(TestBeanWithDep {
                    dyn_dep: ctx.get_bean("testBean")?,
                });
                Ok(bean)
            }))
            .build();
        ctx.register(bean_def)?;

        let bean_dep = ctx.get_bean::<TestBean>("testBean")?;
        assert_eq!(bean_dep.name, "instance_of_testBean");

        let bean = ctx.get_bean::<TestBeanWithDep>("testBeanWithDep")?;
        assert_eq!(bean.dyn_dep.name(), "instance_of_testBean");
        Ok(())
    }
}