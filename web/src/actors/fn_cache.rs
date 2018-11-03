
use collections::fn_cache::FnCache;
use std::sync::Arc;
use actix::prelude::*;
use fn_search_backend_db::models::Function;
use actix::dev::{MessageResponse, ResponseChannel};

pub enum Request {
    UpdateCache(Vec<Function>),
    GetCache,
}

impl Message for Request {
    type Result = Response;
}

pub enum Response {
    Ok,
    Cache(Arc<FnCache>),
}

impl<A, M> MessageResponse<A, M> for Response
    where
        A: Actor,
        M: Message<Result = Response>,
{
    fn handle<R: ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

pub struct FnCacheBroker {
    cache: Arc<FnCache>,
}

impl FnCacheBroker {
    pub fn new(fns: Vec<Function>) -> Self {
        FnCacheBroker{
            cache: Arc::new(fns.iter().collect()),
        }
    }
}

impl Actor for FnCacheBroker {
    type Context = Context<Self>;
}

impl Handler<Request> for FnCacheBroker {
    type Result = Response;

    fn handle(&mut self, req: Request, _ctx: &mut Context<Self>) -> Response {
        match req {
            Request::GetCache => Response::Cache(self.cache.clone()),
            Request::UpdateCache(fns) => {
                self.cache = Arc::new(fns.iter().collect());
                Response::Ok
            },
        }
    }
}
