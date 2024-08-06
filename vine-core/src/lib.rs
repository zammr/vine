use log::trace;

use crate::config::get_config_context;
use crate::context::auto_register_context::get_auto_register_context;
use crate::core::Error;
use crate::core::runner::Runner;
use crate::logger::init_logger;

pub mod core;
pub mod context;
pub mod app;
pub mod logger;
pub mod config;

pub async fn vine_run() -> Result<(), Error> {
    init_logger()?;

    trace!("Setup - create App");
    let app = app::App::default();

    let context = get_config_context()?;
    trace!("Setup - adding {} to App", &context);
    app.add_context(context);

    let context = get_auto_register_context()?;
    trace!("Setup - adding {} to App", &context);
    app.add_context(context);

    trace!("Setup - run App");
    app.run().await
}