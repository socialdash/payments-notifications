mod error;

use std::sync::Arc;

use failure::Fail;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use models::*;
use serde::Deserialize;
use serde_json;

pub use self::error::*;
use super::HttpClient;
use utils::read_body;

pub trait CallbackClient: Send + Sync + 'static {
    fn send(&self, callback: Callback) -> Box<Future<Item = (), Error = Error> + Send>;
}

#[derive(Clone)]
pub struct CallbackClientImpl {
    cli: Arc<HttpClient>,
}

impl CallbackClientImpl {
    pub fn new<C: HttpClient>(cli: C) -> Self {
        Self { cli: Arc::new(cli) }
    }

    fn exec_query<T: for<'de> Deserialize<'de> + Send>(&self, url: String, body: String) -> impl Future<Item = T, Error = Error> + Send {
        let query = url.clone();
        let query1 = query.clone();
        let query2 = query.clone();
        let query3 = query.clone();
        let cli = self.cli.clone();
        let mut builder = Request::builder();
        builder.uri(query).method(Method::POST);
        builder
            .body(Body::from(body))
            .map_err(ectx!(ErrorSource::Hyper, ErrorKind::MalformedInput => query3))
            .into_future()
            .and_then(move |req| cli.request(req).map_err(ectx!(ErrorKind::Internal => query1)))
            .and_then(move |resp| read_body(resp.into_body()).map_err(ectx!(ErrorSource::Hyper, ErrorKind::Internal => query2)))
            .and_then(|bytes| {
                let bytes_clone = bytes.clone();
                String::from_utf8(bytes).map_err(ectx!(ErrorSource::Utf8, ErrorKind::Internal => bytes_clone))
            }).and_then(|string| serde_json::from_str::<T>(&string).map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => string)))
    }
}

impl CallbackClient for CallbackClientImpl {
    fn send(&self, callback: Callback) -> Box<Future<Item = (), Error = Error> + Send> {
        let client = self.clone();
        let url = callback.url.clone();
        Box::new(
            serde_json::to_string(&callback)
                .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => callback))
                .into_future()
                .and_then(move |callback| client.exec_query::<()>(url, callback)),
        )
    }
}

#[derive(Default)]
pub struct CallbackClientMock;

impl CallbackClient for CallbackClientMock {
    fn send(&self, _callback: Callback) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(Ok(()).into_future())
    }
}