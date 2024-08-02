use std::any::{Any, type_name, TypeId};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;


use dashmap::DashMap;
use lazy_static::lazy_static;
use log::{trace, warn};


use crate::core::{DynBean, Error};

pub struct Type {
    id: TypeId,
    name: &'static str,
    downcast_fns: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>
}

lazy_static!(
    static ref TYPES: Arc<DashMap<TypeId, Arc<Type>>> = Arc::new(DashMap::new());
);

impl Type {
    pub fn of<T: 'static>() -> Arc<Type> {
        let type_id = TypeId::of::<T>();
        let ref_mut = TYPES.entry(type_id).or_insert_with(|| {
            let name = type_name::<T>();

            trace!("register Type({})", name);
            Arc::new(Type {
                id: TypeId::of::<T>(),
                name,
                downcast_fns: Default::default(),
            })
        });

        ref_mut.clone()
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn id(&self) -> &TypeId {
        &self.id
    }

    pub fn assignable(&self, type_id: &TypeId) -> bool {
        self.downcast_fns.contains_key(type_id)
    }


    pub fn add_downcast<T: ?Sized + 'static>(&self, downcast_fn: fn(DynBean) -> Result<Arc<T>, DynBean>) {
        let alias_id = TypeId::of::<T>();
        trace!("register {} downcast fn for {}", self, type_name::<T>());
        if let Some(_) = self.downcast_fns.insert(alias_id, Arc::new(downcast_fn)) {
            warn!("override {} downcast fn for {}", self, type_name::<T>());
        }
    }

    pub fn downcast<T: ?Sized + 'static>(dyn_bean: DynBean) -> Result<Arc<T>, Error> {
        let type_id = dyn_bean.as_ref().type_id();
        let Some(type_ref) = TYPES.get(&type_id) else {
            return Err(Error::from("{:?} is not registered in vine Type System"))
        };

        let alias_id = TypeId::of::<T>();
        let Some(downcast_fn) = type_ref.value().downcast_fns.get(&alias_id) else {
            return Err(Error::from("TODO 111"))
        };

        let arc = downcast_fn.clone()
            .downcast::<fn(DynBean) -> Result<Arc<T>, DynBean>>()
            .unwrap();

        let Ok(bean) = (arc.as_ref())(dyn_bean) else {
            return Err(Error::from("TODO 2"))
        };

        Ok(bean)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type({})", self.name)
    }
}


impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Type(id={:?}, name={:?}, downcast_fns.len()={})", &self.id, self.name, self.downcast_fns.len())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::DynBean;
    use crate::core::ty::Type;

    struct TestBean { name: &'static str, }
    trait TestTrait { fn name(&self) -> &'static str; }
    impl TestTrait for TestBean {
        fn name(&self) -> &'static str { self.name }
    }

    #[test]
    fn should_be_thread_safe() {
        let ty = Type::of::<TestBean>();
        ty.add_downcast::<TestBean>(|b| { Ok(Arc::downcast::<TestBean>(b)?)});
        ty.add_downcast::<dyn TestTrait + Sync + Send>(|b| { Ok(Arc::downcast::<TestBean>(b)?)});

        let given_dyn_bean: DynBean = Arc::new(TestBean { name: "test_bean" });

        let test_bean = Type::downcast::<TestBean>(given_dyn_bean.clone()).unwrap();
        let test_trait = Type::downcast::<dyn TestTrait + Send + Sync>(given_dyn_bean.clone()).unwrap();

        assert_eq!(test_trait.name(), test_bean.name);
    }
}
