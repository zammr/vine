use log::trace;

use crate::context::auto_register_context::get_auto_register_context;
use crate::core::Error;
use crate::core::runner::Runner;

pub mod core;
pub mod context;
pub mod app;

pub async fn vine_run() -> Result<(), Error> {
    trace!("Setup - create default App");
    let app = app::App::default();

    trace!("Setup - create auto registered Context");
    let context = get_auto_register_context()?;

    trace!("Setup - add {} to App", &context);
    app.add_context(context);

    trace!("Setup - run App");
    app.run().await
}