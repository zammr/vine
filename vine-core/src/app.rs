use std::sync::Arc;

use async_trait::async_trait;
use log::debug;

use crate::context::context::Context;
use crate::core::Error;
use crate::core::runner::Runner;

pub struct App {
    context: Arc<Context>,
}

impl Default for App {
    fn default() -> Self {
        App {
            context: Arc::new(Context::new("root")),
        }
    }
}

impl App {
    pub fn get_context(&self) -> &Context {
        &self.context
    }

    pub fn add_context(&self, context: Context) {
        self.context.add_context(context);
    }
}

#[async_trait]
impl Runner for App {
    async fn run(&self) -> Result<(), Error> {
        let mut runners = self.context.get_beans::<dyn Runner + Send + Sync>()?;
        debug!("App - initialize {} runners", runners.len());

        self.context.init_beans().await?;

        // TODO: missed feature(sequential, concurrent runners) run each runner in separate thread
        debug!("App - sequentially run");
        while let Some(runner) = runners.pop() {
            runner.run().await?;
        }

        debug!("App - successfully finished in {} ms", "TODO");
        Ok(())
    }
}