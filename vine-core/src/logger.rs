use log4rs::config::RawConfig;
use crate::context::context::Context;
use crate::core::Error;

pub fn init_logger(config: &Context) -> Result<(), Error> {
    let config = config.get_bean::<config::Config>("config")?;
    let raw_config = config.get::<RawConfig>("logging")
        .unwrap_or_else(|_| RawConfig::default());

    log4rs::init_raw_config(raw_config).map_err(|_e| {
        Error::from(r#"
        TODO: doc - logger init error
        "#)
    })
}