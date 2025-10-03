use log4rs::config::RawConfig;
use serde::Deserialize;
use crate::context::context::Context;
use crate::core::Error;

const DEFAULT_LOGGING_CONFIG: &str = r#"
{
    "appenders": {
        "stdout": {
            "kind": "console",
            "encoder": {
                "pattern": "{d(%Y-%m-%dT%H:%M:%S%.3fZ)} {pid} --- [{T:15.15}] {h({l:>5.5})} {M}: {m}{n}"
            }
        }
    },
    "root": {
        "level": "info",
        "appenders": ["stdout"]
    }
}
"#;

pub fn init_logger(config: &Context) -> Result<(), Error> {
    let config = config.get_bean::<config::Config>("config")
        .map_err(|e| Error::from(format!("Failed to get config bean for logger initialization: {}", e)))?;

    let raw_config_value = config.get::<serde_json::Value>("logging")
        .unwrap_or_else(|e| {
            log::debug!("No custom logging configuration found ({}), using default configuration", e);
            serde_json::from_str::<serde_json::Value>(DEFAULT_LOGGING_CONFIG)
                .expect("Failed to parse default logging configuration")
        });

    let raw_config = RawConfig::deserialize(raw_config_value)
        .map_err(|e| {
            log::warn!("Failed to deserialize logging configuration: {}, using default configuration", e);
            e
        })
        .unwrap_or_else(|_| RawConfig::default());

    log4rs::init_raw_config(raw_config).map_err(|e| {
        Error::from(format!("Failed to initialize log4rs logger: {}", e))
    })
}