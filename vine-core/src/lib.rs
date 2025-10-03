use log::trace;

use crate::app::App;
use crate::config::get_config_context;
use crate::context::auto_register_context::get_auto_register_context;
use crate::core::Error;
use crate::logger::init_logger;

pub mod core;
pub mod context;
pub mod app;
pub mod logger;
pub mod config;

pub fn create_app() -> Result<App, Error> {
    let context = get_config_context(vec![
        "app.yaml".to_string(),
        "app.yml".to_string(),
    ])?;
    init_logger(&context)?;

    trace!("setup - create default App instance");
    let app = App::default();

    trace!("setup - adding {} to App", &context);
    app.add_context(context);

    let context = get_auto_register_context()?;
    trace!("setup - adding {} to App", &context);
    app.add_context(context);

    Ok(app)
}
