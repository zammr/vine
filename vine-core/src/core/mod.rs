use std::any::Any;
use std::sync::Arc;

pub mod ty;

pub type Error = String;
pub(crate) type DynBean = Arc<dyn Any + Send + Sync>;