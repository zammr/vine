mod property_resolver;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use config::{Config, Environment, File};
use crate::context::context::Context;
use crate::core::bean_def::BeanDef;
use crate::core::Error;
use crate::core::ty::Type;

pub use property_resolver::PropertyResolver as PropertyResolver;

pub fn get_config_context(config_files: Vec<String>) -> Result<Context, Error> {
    let config_context = Context::new("config");

    let ty = Type::of::<Config>();
    ty.add_downcast::<Config>(|b| Ok(Arc::downcast::<Config>(b)?));
    ty.add_downcast::<dyn PropertyResolver + Send + Sync>(|b| Ok(Arc::downcast::<Config>(b)?));

    let mut config_builder = Config::builder();
    for config_file in config_files {
        if Path::new(config_file.as_str()).exists() {
            log::debug!("Loading config file: {}", config_file);
            config_builder = config_builder.add_source(File::with_name(&config_file))
        }
    }

    // Use environment variables with APP prefix (e.g., APP_server_port=8080)
    log::debug!("Loading environment variables with APP prefix");
    config_builder = config_builder.add_source(
        Environment::with_prefix("APP").prefix_separator("_").separator(".").try_parsing(true)
    );

    // Parse -- style command line arguments (e.g., --server.port=8080)
    log::debug!("Parsing -- style command line arguments");
    let d_args: HashMap<String, String> = std::env::args()
        .filter_map(|arg| {
            if let Some(stripped) = arg.strip_prefix("--") {
                stripped.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))
            } else {
                None
            }
        })
        .collect();

    for (key, value) in d_args {
        config_builder = config_builder.set_override(key, value).map_err(|e| {
            Error::from(format!("configuration error: {:#?}", e))
        })?;
    }

    let config = Arc::new(config_builder.build().map_err(|e| {
        Error::from(format!("configuration error: {:#?}", e))
    })?);
    config_context.register(BeanDef::builder()
        .name("config")
        .ty(ty)
        .get(Arc::new(move |_| { Ok(config.clone()) }))
        .build())?;

    Ok(config_context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PropertyResolver;

    #[test]
    fn should_allow_config_downcast_to_property_resolver() {
        let context = get_config_context(vec![]).unwrap();
        let resolver = context.get_bean::<dyn PropertyResolver + Send + Sync>("config");
        assert!(resolver.is_ok());
    }

    #[test]
    fn should_load_environment_variables() {
        std::env::set_var("APP_test_key", "test_value");

        let context = get_config_context(vec![]).unwrap();
        let resolver = context.get_bean::<dyn PropertyResolver + Send + Sync>("config").unwrap();
        assert_eq!(resolver.get_string("test_key"), Some("test_value".to_string()));

        std::env::remove_var("APP_test_key");
    }

    #[test]
    fn should_compute_template_values() {
        std::env::set_var("APP_prop1", "value1");
        std::env::set_var("APP_prop2", "value2");

        let context = get_config_context(vec![]).unwrap();
        let resolver = context.get_bean::<dyn PropertyResolver + Send + Sync>("config").unwrap();

        let result = resolver.compute_template_value("${prop1}_${prop2}").unwrap();
        assert_eq!(result, "value1_value2");

        std::env::remove_var("APP_prop1");
        std::env::remove_var("APP_prop2");
    }

    #[test]
    fn should_compute_template_values_with_defaults() {
        let context = get_config_context(vec![]).unwrap();
        let resolver = context.get_bean::<dyn PropertyResolver + Send + Sync>("config").unwrap();

        let result = resolver.compute_template_value("${missing_prop:default_value}").unwrap();
        assert_eq!(result, "default_value");
    }

    #[test]
    fn should_compute_typed_template_values() {
        std::env::set_var("APP_server.port", "8080");
        std::env::set_var("APP_enabled", "true");

        let context = get_config_context(vec![]).unwrap();
        let resolver = context.get_bean::<dyn PropertyResolver + Send + Sync>("config").unwrap();

        assert_eq!(resolver.compute_template_value_as_u16("${server.port}").unwrap(), 8080);
        assert_eq!(resolver.compute_template_value_as_bool("${enabled}").unwrap(), true);

        std::env::remove_var("APP_server_port");
        std::env::remove_var("APP_enabled");
    }
}
