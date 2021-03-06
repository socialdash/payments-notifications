mod error;
mod requests;

use std::sync::Arc;

use failure::Fail;
use futures::prelude::*;
use hyper::Method;
use hyper::{Body, Request};
use serde_json;

pub use self::error::*;
use self::requests::SendGridPayload;
use super::HttpClient;
use config::Config;
use models::*;

pub trait EmailClient: Send + Sync + 'static {
    fn send(&self, email: Email) -> Box<Future<Item = (), Error = Error> + Send>;
}

#[derive(Clone)]
pub struct EmailClientImpl {
    cli: Arc<HttpClient>,
    api_addr: String,
    api_key: String,
    send_mail_path: String,
    from_email: String,
    from_name: String,
}

impl EmailClientImpl {
    pub fn new<C: HttpClient>(config: &Config, cli: C) -> Self {
        Self {
            cli: Arc::new(cli),
            api_addr: config.sendgrid.api_addr.clone(),
            api_key: config.sendgrid.api_key.clone(),
            send_mail_path: config.sendgrid.send_mail_path.clone(),
            from_email: config.sendgrid.from_email.clone(),
            from_name: config.sendgrid.from_name.clone(),
        }
    }

    fn exec_query(&self, email: Email) -> impl Future<Item = (), Error = Error> + Send {
        let api_addr = self.api_addr.clone();
        let api_key = self.api_key.clone();
        let send_mail_path = self.send_mail_path.clone();
        let from_email = self.from_email.clone();
        let from_name = self.from_name.clone();
        let payload = SendGridPayload::from_email(email, from_email, from_name);
        let query = format!("{}/{}", api_addr.clone(), send_mail_path.clone());
        let query1 = query.clone();
        let query3 = query.clone();
        let cli = self.cli.clone();
        let mut builder = Request::builder();
        builder
            .uri(query)
            .method(Method::POST)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json");

        serde_json::to_string(&payload)
            .map_err(ectx!(ErrorSource::Json, ErrorKind::Internal => payload))
            .into_future()
            .and_then(move |body| {
                builder
                    .body(Body::from(body))
                    .map_err(ectx!(ErrorSource::Hyper, ErrorKind::MalformedInput => query3))
                    .into_future()
            })
            .and_then(move |req| cli.request(req).map_err(ectx!(ErrorKind::Internal => query1)))
            .map(|_| ())
    }
}

impl EmailClient for EmailClientImpl {
    fn send(&self, email: Email) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(self.exec_query(email))
    }
}

#[derive(Default)]
pub struct EmailClientMock;

impl EmailClient for EmailClientMock {
    fn send(&self, _email: Email) -> Box<Future<Item = (), Error = Error> + Send> {
        Box::new(Ok(()).into_future())
    }
}
