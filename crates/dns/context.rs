use moka::future::Cache;

pub struct ServerContext {
    pub cache: Cache<u16, u16>,
}
