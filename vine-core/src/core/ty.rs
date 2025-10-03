use std::any::{Any, type_name, TypeId};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;


use dashmap::DashMap;
use lazy_static::lazy_static;
use log::{trace, warn};


use crate::core::{DynBean, Error};

/// A runtime type descriptor that enables dynamic type operations and trait object downcasting.
/// 
/// The Type system provides:
/// - Runtime type identification and registration
/// - Dynamic downcasting to concrete types and trait objects
/// - Thread-safe type operations through DashMap
/// 
/// Types are registered automatically when first accessed via `Type::of::<T>()`.
pub struct Type {
    /// Unique identifier for this type
    id: TypeId,
    /// Human-readable type name
    name: &'static str,
    /// Registry of downcast functions for converting to different types/traits
    downcast_fns: Arc<DashMap<TypeId, Arc<dyn Any + Send + Sync>>>
}

// Global registry of all registered types, indexed by TypeId for fast lookup
lazy_static!(
    static ref TYPES: Arc<DashMap<TypeId, Arc<Type>>> = Arc::new(DashMap::new());
);

impl Type {
    /// Gets or creates a Type descriptor for the given type T.
    /// 
    /// This method is thread-safe and will only create one Type instance per type.
    /// The type is automatically registered in the global TYPES registry.
    ///
    /// # Examples
    /// ```
    /// use vine_core::core::ty::Type;
    ///
    /// let string_type = Type::of::<String>();
    /// let int_type = Type::of::<i32>();
    /// ```
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

    /// Returns the human-readable name of this type.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Returns the unique TypeId for this type.
    pub fn id(&self) -> &TypeId {
        &self.id
    }

    /// Checks if this type can be downcast to the type identified by the given TypeId.
    /// 
    /// Returns true if a downcast function has been registered for the target type.
    /// This is used to determine type compatibility at runtime.
    pub fn assignable(&self, type_id: &TypeId) -> bool {
        self.downcast_fns.contains_key(type_id)
    }


    /// Registers a downcast function for converting DynBean instances to type T.
    /// 
    /// This enables runtime conversion from the current type to trait objects or other types.
    /// If a downcast function already exists for type T, it will be replaced with a warning.
    /// 
    /// # Arguments
    /// * `downcast_fn` - Function that attempts to downcast a DynBean to Arc<T>
    ///
    /// # Examples
    /// ```ignore
    /// use vine_core::core::ty::Type;
    /// use std::sync::Arc;
    ///
    /// let ty = Type::of::<MyStruct>();
    /// ty.add_downcast::<dyn MyTrait>(|bean| Ok(Arc::downcast::<MyStruct>(bean)?));
    /// ```
    pub fn add_downcast<T: ?Sized + 'static>(&self, downcast_fn: fn(DynBean) -> Result<Arc<T>, DynBean>) {
        let alias_id = TypeId::of::<T>();
        trace!("register {} downcast fn for {}", self, type_name::<T>());
        if let Some(_) = self.downcast_fns.insert(alias_id, Arc::new(downcast_fn)) {
            warn!("override {} downcast fn for {}", self, type_name::<T>());
        }
    }

    /// Attempts to downcast a DynBean to the specified type T.
    /// 
    /// This method looks up the appropriate downcast function and applies it to convert
    /// the DynBean to the requested type. The target type must have been registered
    /// via `add_downcast` on the source type.
    /// 
    /// # Arguments
    /// * `dyn_bean` - The dynamic bean to downcast
    ///
    /// # Returns
    /// * `Ok(Arc<T>)` - Successfully downcast bean
    /// * `Err(Error)` - If the type is not registered or downcast fails
    ///
    /// # Examples
    /// ```ignore
    /// use vine_core::core::ty::Type;
    /// use vine_core::core::bean_def::DynBean;
    /// use std::sync::Arc;
    ///
    /// let bean: DynBean = Arc::new(MyStruct::new());
    /// let result: Arc<MyStruct> = Type::downcast(bean)?;
    /// ```
    pub fn downcast<T: ?Sized + 'static>(dyn_bean: DynBean) -> Result<Arc<T>, Error> {
        let type_id = dyn_bean.as_ref().type_id();
        let Some(type_ref) = TYPES.get(&type_id) else {
            return Err(Error::from(format!("Type {:?} is not registered in vine Type System", type_id)))
        };

        let alias_id = TypeId::of::<T>();
        let Some(downcast_fn) = type_ref.value().downcast_fns.get(&alias_id) else {
            return Err(Error::from(format!("No downcast function registered for {} -> {}", type_ref.value().name(), type_name::<T>())))
        };

        let arc = downcast_fn.clone()
            .downcast::<fn(DynBean) -> Result<Arc<T>, DynBean>>()
            .unwrap();

        let Ok(bean) = (arc.as_ref())(dyn_bean) else {
            return Err(Error::from(format!("Failed to downcast {} to {}", type_ref.value().name(), type_name::<T>())))
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
