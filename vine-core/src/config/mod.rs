mod property_resolver;

use std::path::Path;
use std::sync::Arc;
use config::{Config, Environment, File};
use crate::context::context::Context;
use crate::core::bean_def::BeanDef;
use crate::core::Error;
use crate::core::ty::Type;

pub use property_resolver::PropertyResolver as PropertyResolver;

pub fn get_config_context() -> Result<Context, Error> {
    let config_context = Context::new("config");

    let ty = Type::of::<Config>();
    ty.add_downcast::<Config>(|b| Ok(Arc::downcast::<Config>(b)?));
    ty.add_downcast::<dyn PropertyResolver + Send + Sync>(|b| Ok(Arc::downcast::<Config>(b)?));

    let mut config_builder = Config::builder();

    if Path::new("app.yaml").exists() {
        config_builder = config_builder.add_source(File::with_name("app.yaml"))
    }
    config_builder = config_builder.add_source(Environment::with_prefix("-D"));

    let config = Arc::new(config_builder.build().unwrap());
    config_context.register(BeanDef::builder()
        .name("config")
        .ty(ty)
        .get(Arc::new(move |_| { Ok(config.clone()) }))
        .build())?;

    Ok(config_context)
}
