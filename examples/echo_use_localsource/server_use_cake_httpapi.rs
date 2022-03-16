use cakerabbit_core::{serve, Service, ServiceWithClient, CakeResult, CakeServiceServe};
use std::pin::Pin;
use futures::{Future, future, TryFutureExt};
use cakerabbit_core::{Value};
use std::{io, result};
use std::net::{SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use std::io::Error;
use tokio_util::compat::TokioAsyncReadCompatExt;
use env_logger::Env;
use serde::{Serialize, Deserialize};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder,
                middleware::Logger, Result};
use actix_web::dev::Server;

const SVC_NAME: &str = "EchoRs";

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
struct SayHelloReply {
  name: String,
}

// todo: call by client
fn say_hello(params: &[Value]) -> CakeResult<Vec<u8>> {
  println!("say_hello params -->>> {:?}", params);
  if let Value::String(ref value) = params[0] {
    if let Some(val) = value.as_str() {
      let rsp = SayHelloReply {
        // name: "foo".to_string()
        name: val.to_string()
      };
      let rsp_vec = serde_json::to_vec(&rsp).unwrap();
      return Ok(rsp_vec);
    }
  }

  Ok(Vec::from("wrong param".to_string()))
}

#[derive(Deserialize)]
struct Hello {
    name: String,
}

// todo: call by http restful api
async fn say_hello_api(hello: web::Json<Hello>) -> impl Responder {
  // let hello_vec = say_hello(&["foo".into()]).unwrap();

  let name = &hello.name;
  let namex = name.to_string();
  let hello_vec = say_hello(&[namex.into()]).unwrap();

  let rsp = String::from_utf8(hello_vec).unwrap();
  HttpResponse::Ok()
    .content_type("application/json").body(rsp)
}

#[actix_web::main]
async fn http_app(http_addr: &'static str) -> io::Result<()> {
  HttpServer::new(|| {
    App::new()
      .route("/hello", web::get().to(say_hello_api))
      .wrap(Logger::default())
  }).workers(2)
    .bind(http_addr)?
    .run().await
}

#[tokio::main]
async fn main() -> io::Result<()> {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

  let http_addr  = "0.0.0.0:8089";
  let mut svc_serve = CakeServiceServe::new(SVC_NAME.to_string(),
                                            "pomid/".to_string(),
                                            "0.0.0.0:9527".to_string(),
                                            "consul".to_string(),
                                            "consul_test:8500".to_string(),
                                            "1m0s".to_string(),
                                            false,
                                            http_addr);

  // todo: register svc method
  svc_serve.register_fn("say_hello".into(), say_hello);

  let svc_serv_cl = svc_serve.clone();
  tokio::task::spawn(async move {
    println!("=== enable http ===");
    http_app(http_addr);
    // svc_serv_cl.run_http(http_app);
  });

  // todo: run
  svc_serve.run().await;

  Ok(())
}

