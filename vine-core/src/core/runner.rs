use async_trait::async_trait;
use crate::core::Error;

#[async_trait]
pub trait Runner {
    async fn run(&self) -> Result<(), Error>;
}
