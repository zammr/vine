use std::any::type_name_of_val;
use std::sync::Arc;
use async_trait::async_trait;
use config::Config;
use log::debug;
use tokio::runtime::Runtime;
use crate::core::Error;

#[async_trait]
pub trait Runner {
    fn name(&self) -> &str {
        let name = type_name_of_val(self);
        match name.rsplit_once("::") {
            None => name,
            Some((_, name)) => name
        }
    }

    fn runtime(&self, _config: Arc<Config>) -> Result<Runtime, Error> {
        debug!("create tokio Runtime for {} runner", self.name());
        tokio::runtime::Builder::new_multi_thread()
            .thread_name(self.name())
            .build()
            .map_err(|_e| Error::from(format!("cannot initialize runtime for runner {}", self.name())))
    }

    async fn run(&self) -> Result<(), Error>;
}
