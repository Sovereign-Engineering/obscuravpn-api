use hyper_util::client::legacy::connect::dns::GaiResolver;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tower_service::Service;

pub trait ResolverFallbackCache: Send + Sync {
    fn get(&self, name: &str) -> Vec<SocketAddr>;
    fn set(&self, name: &str, addr: &[SocketAddr]);
}

pub(crate) struct GaiResolverWithFallback {
    gai: GaiResolver,
    cache: Arc<dyn ResolverFallbackCache>,
}

impl GaiResolverWithFallback {
    pub fn new(cache: Arc<dyn ResolverFallbackCache>) -> Self {
        Self {
            gai: GaiResolver::new(),
            cache,
        }
    }
}

impl Resolve for GaiResolverWithFallback {
    fn resolve(&self, name: Name) -> Resolving {
        let mut gai = self.gai.clone();
        let cache = self.cache.clone();
        Box::pin(async move {
            let name_str = name.as_str();
            let name_hyper = hyper_util::client::legacy::connect::dns::Name::from_str(name_str).map_err(|error| {
                tracing::error!(message_id = "9hJXKXwo", ?name, ?error, "invalid domain name: {}", error);
                Box::<dyn std::error::Error + Send + Sync>::from(error)
            })?;
            let gai_result = gai.call(name_hyper).await;
            let error = match gai_result {
                Ok(addrs) => {
                    let addrs: Vec<SocketAddr> = addrs.into_iter().collect();
                    tracing::info!(message_id = "kpmHpAo5", ?name, ?addrs, "adding addresses for domain to cache");
                    cache.set(name_str, &addrs);
                    return Ok(Box::new(addrs.into_iter()) as Addrs);
                }
                Err(error) => {
                    tracing::error!(message_id = "XfuftAp2", ?name, ?error, "failed to resolve: {}", error);
                    Box::from(error)
                }
            };
            let addrs = cache.get(name_str);
            if addrs.is_empty() {
                tracing::error!(message_id = "aVGK6zm8", ?name, "no addresses cached for domain name");
                return Err(error);
            }
            tracing::info!(message_id = "zgzilM7M", ?name, ?addrs, "found addresses for domain in cache");
            Ok(Box::new(addrs.into_iter()))
        })
    }
}

pub(crate) struct NoResolverFallbackCache();
impl ResolverFallbackCache for NoResolverFallbackCache {
    fn get(&self, _name: &str) -> Vec<SocketAddr> {
        Vec::new()
    }
    fn set(&self, _name: &str, _addr: &[SocketAddr]) {}
}
