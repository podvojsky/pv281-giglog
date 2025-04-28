use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct PoolHandler {
    pg_pool: Arc<PgPool>,
}

impl PoolHandler {
    pub fn new(pg_pool: Arc<PgPool>) -> Self {
        Self { pg_pool }
    }
    #[allow(dead_code)]
    pub async fn disconnect(&mut self) {
        self.pg_pool.close().await;
    }

    pub fn pool(&self) -> &PgPool {
        &self.pg_pool
    }
}
