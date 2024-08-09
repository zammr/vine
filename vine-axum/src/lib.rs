use std::sync::Arc;
use async_trait::async_trait;
use axum::Router;
use axum::routing::MethodRouter;
use dashmap::DashMap;
use linkme::distributed_slice;
use log::{debug, trace};
use vine_core::config::PropertyResolver;
use vine_core::context::context::Context;
use vine_core::core::Error;
use vine_core::context::auto_register_context::SETUP;
use vine_core::core::bean_def::BeanDef;
use vine_core::core::ty::Type;

pub struct Web {
    host: String,
    port: String,
    routes: DashMap<String, Vec<MethodRouter>>,
}

impl Web {
    pub fn add_route(&self, path: String, method_router: MethodRouter) {
        if let Some(mut r) = self.routes.get_mut(&path) {
            r.push(method_router);
        } else {
            self.routes.insert(path, vec![method_router]);
        }
    }
}

#[async_trait]
impl vine_core::core::runner::Runner for Web {
    async fn run(&self) -> Result<(), Error> {
        trace!("register endpoints");
        let router = {
            let mut router = Router::new();
            for route in &self.routes {
                let method_router = route.value().clone().into_iter().reduce(|acc, h| acc.merge(h));
                if let Some(method_router) = method_router {
                    trace!("register endpoint: {}", route.key());
                    router = router.route(route.key(), method_router);
                }
            }
            router
        };

        let addr = format!("{}:{}", &self.host, &self.port);

        trace!("tokio::net::TcpListener bind address: {}", &addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

        debug!("axum::serve: {}", &addr);
        axum::serve(listener, router).await.unwrap();
        Ok(())
    }
}

#[distributed_slice(SETUP)]
pub static SETUP_WEB: fn(&Context) -> Result<(), Error> = |ctx| {
    trace!("Setup axum based web Bean");
    let ty = Type::of::<Web>();
    ty.add_downcast::<Web>(|b| Ok(Arc::downcast::<Web>(b)?));
    ty.add_downcast::<dyn vine_core::core::runner::Runner + Send + Sync>(|b| Ok(Arc::downcast::<Web>(b)?));

    ctx.register(BeanDef::builder()
        .name("web")
        .ty(ty)
        .get(Arc::new(|ctx| {
            let config = ctx.get_primary_bean::<dyn PropertyResolver + Send + Sync>()?;
            let host = config.compute_template_value("${server.host:0.0.0.0}")?;
            let port = config.compute_template_value("${server.port:3000}")?;
            let web = Arc::new(Web { host, port, routes: Default::default() });
            Ok(web)
        }))
        .build())
};