# Vine

Vine is rust framework inspired by Spring Boot

Example:
```rust
use std::sync::Arc;
use async_trait::async_trait;
use axum::extract::Path;
use axum::response::IntoResponse;
use vine::{Bean, controller, get, injectable};
use vine::vine_core::core::Error;

#[async_trait]
trait Service {
    async fn compute(&self, name: String) -> String;
}

#[derive(Bean)]
struct Controller {
    service: Arc<dyn Service + Sync + Send>,
}

#[controller]
impl Controller {
    #[get("/hello/:name")]
    async fn say_hello(&self, Path(name): Path<String>) -> impl IntoResponse {
        self.service.compute(name).await
    }
}

#[derive(Bean)]
struct ServiceImpl {
    #[value("${service.name:DefaultName}")] name: String,
}

#[async_trait]
#[injectable]
impl Service for ServiceImpl {
    async fn compute(&self, name: String) -> String {
        format!("Hello from the {}: {}", &self.name, name)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    vine::create_app()?
        .exec().await
}
```
