use crate::collections::FnCache;
use actix_web::*;
use fn_search_backend_db::diesel::pg::PgConnection;
use parking_lot::RwLock;
use r2d2::Error as R2D2Error;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use std::sync::Arc;

pub type PoolConn = PooledConnection<ConnectionManager<PgConnection>>;
pub type PoolConnRes = Result<PoolConn, R2D2Error>;

pub struct AppState {
    pool: Pool<ConnectionManager<PgConnection>>,
    cache: RwLock<Arc<FnCache>>,
}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>, cache: Arc<FnCache>) -> Self {
        AppState {
            pool,
            cache: RwLock::new(cache),
        }
    }

    /// to convert result into PgConnection, deref then ref
    /// ```
    /// &*state.db_conn()?
    /// ```
    pub fn db_conn(&self) -> PoolConnRes {
        self.pool.get()
    }

    pub fn get_fn_cache(&self) -> Arc<FnCache> {
        self.cache.read().clone()
    }

    pub fn update_fn_cache(&self, c: FnCache) {
        *self.cache.write() = Arc::new(c);
    }
}
