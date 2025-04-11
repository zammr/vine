use std::sync::Arc;
use std::time::Instant;

use log::{debug, info, warn};

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

    pub async fn exec(&self) -> Result<(), Error> {
        let timer = Instant::now();
        
        info!("starting application");
        self.context.init_contexts()?;

        let mut runners = self.context.get_beans::<dyn Runner + Send + Sync>()?;
        debug!("starting {} runners", runners.len());

        let mut handles = Vec::new();
        while let Some(r) = runners.pop() {
            let runner = r.clone();
            let name = runner.name().to_string();
            
            debug!("starting runner {}", r.name());
            handles.push(tokio::spawn(async move {
                let result = runner.run().await;
                (name, result)
            }));
            debug!("runner {} has been started in {} micros", r.name(), timer.elapsed().as_micros());
        }
        info!("started in {} micros", timer.elapsed().as_micros());

        let mut errors = Vec::new();
        while let Some(runner_result) = handles.pop() {
            let (name, result) = runner_result.await.map_err(|_| {
               Error::from("failed to start runner")
            })?;

            if let Err(error) = result {
                warn!("runner {} has been finished with error: {}", &name, &error);
                errors.push(error);
            }
        }

        info!("application finished {} micros", timer.elapsed().as_micros());
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::from(errors.join("\n")))
        }
    }
}