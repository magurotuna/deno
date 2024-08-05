// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use http::{Extensions, Uri};
use hyper_util::client::legacy::connect::{Connection, HttpInfo};
use tower::Service;

#[derive(Debug, Clone)]
pub struct TcpConnInfoConnector<C> {
  pub(crate) inner: C,
  pub(crate) tcp_conn_info_slot: Arc<Mutex<Option<HttpInfo>>>,
}

impl<C> TcpConnInfoConnector<C> {
  pub fn new(inner_connector: C) -> Self {
    Self {
      inner: inner_connector,
      tcp_conn_info_slot: Arc::new(Mutex::new(None)),
    }
  }
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;
type BoxError = Box<dyn std::error::Error + Send + Sync>;

impl<C> Service<Uri> for TcpConnInfoConnector<C>
where
  C: Service<Uri> + Clone + Send + 'static,
  C::Response:
    hyper::rt::Read + hyper::rt::Write + Connection + Unpin + Send + 'static,
  C::Future: Send + 'static,
  C::Error: Into<BoxError> + 'static,
{
  type Response = C::Response;
  type Error = BoxError;
  type Future = BoxFuture<Result<Self::Response, Self::Error>>;

  fn poll_ready(
    &mut self,
    cx: &mut Context<'_>,
  ) -> Poll<Result<(), Self::Error>> {
    self.inner.poll_ready(cx).map_err(Into::into)
  }

  fn call(&mut self, dst: Uri) -> Self::Future {
    let mut inner = self.inner.clone();
    let tcp_conn_info_slot = self.tcp_conn_info_slot.clone();

    Box::pin(async move {
      dbg!("🥶🥶🥶🥶🥶🥶");
      let res = inner.call(dst).await.map_err(Into::into)?;
      dbg!("🥶🥶🥶🥶🥶🥶");
      let connected = res.connected();
      dbg!("🥶🥶🥶🥶🥶🥶");
      let mut exts = Extensions::new();
      connected.get_extras(&mut exts);
      if let Some(info) = exts.get::<HttpInfo>() {
        dbg!(info);
        tcp_conn_info_slot.lock().unwrap().replace(info.clone());
      }
      Ok(res)
    })
  }
}
