use std::any::Any;
use std::sync::Arc;

pub mod ty;
pub mod bean_def;
pub mod runner;

pub type Error = String;
pub(crate) type DynBean = Arc<dyn Any + Send + Sync>;