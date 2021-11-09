use std::{io, result};
use std::net::SocketAddr;
use std::pin::Pin;
use std::str;

use futures::{Future, future, FutureExt, ready, Sink, Stream, TryFutureExt};
use rmpv::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::compat::TokioAsyncReadCompatExt;

use crate::{CakeError, Client, serve};
use crate::reg::{Register, RegisterImpl};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::io::Error;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware::Logger, Result};

pub trait Service: Send {
  type RequestFuture: Future<Output=Result<Value, Value>> + 'static + Send;

  fn handle_request(&mut self, method: &str, params: &[Value]) -> Self::RequestFuture;

  fn handle_notification(&mut self, method: &str, params: &[Value]);
}


pub trait ServiceWithClient {
  type RequestFuture: Future<Output=Result<Value, Value>> + 'static + Send;

  fn handle_request(
    &mut self,
    client: &mut Client,
    method: &str,
    params: &[Value],
  ) -> Self::RequestFuture;

  fn handle_notification(&mut self, client: &mut Client, method: &str, params: &[Value]);
}

impl<S: Service> ServiceWithClient for S {
  type RequestFuture = <S as Service>::RequestFuture;

  fn handle_request(
    &mut self,
    _client: &mut Client,
    method: &str,
    params: &[Value],
  ) -> Self::RequestFuture {
    self.handle_request(method, params)
  }

  fn handle_notification(&mut self, _client: &mut Client, method: &str, params: &[Value]) {
    self.handle_notification(method, params);
  }
}

// #[macro_export]
macro_rules! register_cakefn {
    ($fn_key: expr) => {
      trace!("---register cakefn---");
    };
}

// todo: cakeRabbit wrap service server
#[derive(Clone)]
pub struct CakeServiceServe {
  svc_name: String,
  svc_prefix: String,
  addr: String,
  reg_adapter: String,
  reg_addr: String,
  reg_ttl: String,
  svc_fns: Arc<RwLock<HashMap<String, Box<CakeFn>>>>,
  debug: bool,
}

pub type CakeResult<T> = result::Result<T, CakeError>;

impl From<std::io::Error> for CakeError {
  fn from(err: Error) -> Self {
    // trace!("err ------------ {:?}", err);
    CakeError(err.to_string())
  }
}

pub type CakeFn = fn(&[Value]) -> CakeResult<Vec<u8>>;

impl CakeServiceServe {
  pub fn new(svc_name: String, svc_prefix: String, addr: String,
             reg_adapter: String, reg_addr: String, reg_ttl: String,
             debug: bool,
  ) -> Self {
    CakeServiceServe {
      svc_name,
      svc_prefix,
      addr,
      reg_adapter,
      reg_addr,
      reg_ttl,
      svc_fns: Arc::new(Default::default()),
      debug,
    }
  }

  pub fn register_svc(self) -> Result<bool, CakeError> {
    let svc_split = self.addr.split(":");
    let svc_split_vec: Vec<&str> = svc_split.collect();
    let svc_namex = self.svc_name.clone();
    let mut reg = Register::new_for_service(self.reg_adapter, self.reg_addr,
                                            self.svc_name, self.svc_prefix.to_string(),
                                            svc_split_vec[1].to_string(), self.reg_ttl.to_string(),
                                            self.debug);
    let res = reg.do_reg();
    match res {
      Ok(reg_res) => { info!("Service {} register result {}", svc_namex, reg_res) }
      Err(e) => {
        info!("Service {} register error: {:?}.", svc_namex, e);
        // std::process::exit(0);   // dont need to exit service
      }
    }
    Ok(true)
  }

  pub fn register_fn(&self, fn_key: String, f: CakeFn) {
    let mut fn_map = self.svc_fns.write().unwrap();
    fn_map.insert(fn_key, Box::new(f));
  }

  pub fn cakefn_wrap(fn_key: String) {
    register_cakefn!(fn_key);
  }

  pub async fn run(self) -> io::Result<()> {
    let selfx = self.clone();
    // todo: register svc
    tokio::task::spawn(async move {
      selfx.register_svc();
    });
    // todo: http api
    // tokio::task::spawn(async move {
    //   println!("===starting http api serv===");
    //   enable_httpapi();
    // });

    let addr: SocketAddr = self.addr.parse().unwrap();
    let listener = TcpListener::bind(&addr).await?;
    let mut index = 0;
    loop {
      let socket = match listener.accept().await {
        Ok((socket, _)) => {
          index += 1;
          socket
        }

        Err(e) => {
          error!("error on TcpListener: {}", e);
          continue;
        }
      };

      info!("new client connection -------- {:?}, index: {}", socket, index);
      info!("spawning a new Service Serve");
      // todo: add move to optimize self.clone()??
      // tokio::spawn(serve(socket.compat(), self.clone())
      // tokio::task::spawn(serve(socket.compat(), self.clone())
      tokio::task::spawn(serve(socket.compat(), self.clone())
        .map_err(|e| info!("service start error {}", e)));

      // tokio::task::spawn( async move { serve(socket.compat(), self.clone())
      //   .map_err(|e| info!("service start error {}", e))} );
    }
  }
}

#[actix_web::main]          // 这是一个注解, 类似java的@
async fn enable_httpapi() -> std::io::Result<()> {
  HttpServer::new(|| {
    App::new()
      .service(pong)
      .wrap(Logger::default())
  }).workers(2)
    .bind("127.0.0.1:8089")?
    .run()
    .await
}

#[get("/pong")]
async fn pong() -> impl Responder {
  // let name = "xx";
  HttpResponse::Ok().body("pong")
  // let name = "xx";
}

impl Service for CakeServiceServe {
  type RequestFuture = Pin<Box<dyn Future<Output=Result<Value, Value>> + Send>>;

  fn handle_request(&mut self, method: &str, params: &[Value]) -> Self::RequestFuture {
    info!("get request: {}", method);
    let map = self.svc_fns.read().unwrap();
    match map.get(method) {
      None => { warn!("Service {} not found method {}", self.svc_name, method) }
      Some(box_fn) => {
        let f = **box_fn;
        let rsp_res = f(params);
        let rsp = rsp_res.unwrap();
        trace!("server rsp Vec[u8] ---------- {:?}", rsp);
        let rsp_decode = str::from_utf8(&rsp).unwrap();
        trace!("server rsp  ---------- {:?}", rsp_decode);

        return Box::pin(
          future::ok(rsp_decode.into())
          // future::ok("echo server response!".into())
        );
      }
    }

    Box::pin(future::err("Service handle request error".into()))
  }

  fn handle_notification(&mut self, method: &str, params: &[Value]) {
    info!("get nofify: {}", method);
  }
}

