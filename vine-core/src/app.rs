use std::sync::Arc;
use std::time::{Instant};

use config::Config;
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

        let config = self.context.get_bean::<Config>("config")?;
        let mut runners = self.context.get_beans::<dyn Runner + Send + Sync>()?;

        debug!("starting {} runners", runners.len());
        let mut runner_results = Vec::new();
        while let Some(r) = runners.pop() {
            let config = config.clone();
            let runner = r.clone();
            runner_results.push(tokio::task::spawn_blocking(move || {
                let name = runner.name().to_string();

                debug!("create Runtime for runner {}", &name);
                let runtime = runner.runtime(config).unwrap(); // TODO: fix unwrap

                debug!("start runner {} within runtime", &name);
                let join_handle = runtime.spawn(async move { runner.run().await });
                (name, runtime, join_handle)
            }));
            debug!("runner {} has been started in {} micros", r.name(), timer.elapsed().as_micros());
        }
        info!("started in {} micros", timer.elapsed().as_micros());

        let mut errors = Vec::new();
        while let Some(runner_result) = runner_results.pop() {
            let (name, _runtime, join_handle) = runner_result.await.map_err(|_join_error| {
               Error::from("failed to start runner")
            })?;

            let result = join_handle.await.map_err(|_join_error| {
                Error::from(format!("failed to execute runner {}", &name))
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