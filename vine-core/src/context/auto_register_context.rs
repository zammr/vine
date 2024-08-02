use linkme::distributed_slice;
use log::debug;
use crate::context::context::Context;
use crate::core::Error;

#[distributed_slice]
pub static SETUP: [fn(&Context) -> Result<(), Error>] = [..];

pub fn get_auto_register_context() -> Result<Context, Error> {
    debug!("setup auto registered context");

    let context = Context::new("auto-registered");
    for setup_fn in SETUP {
        (setup_fn)(&context)?;
    }

    Ok(context)
}