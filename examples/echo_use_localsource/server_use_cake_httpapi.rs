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

const SVC_NAME: &str = "EchoRs";

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
struct SayHelloReply {
  name: String,
}

fn say_hello(params: &[Value]) -> CakeResult<Vec<u8>> {
  println!("say_hello params ------------ {:?}", params);
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

#[tokio::main]
async fn main() -> io::Result<()> {
  env_logger::from_env(Env::default().default_filter_or("info")).init();
  let mut svc_serve = CakeServiceServe::new(SVC_NAME.to_string(),
                                            "pomid/".to_string(),
                                            "0.0.0.0:9527".to_string(),
                                            "consul".to_string(),
                                            "consul_test:8500".to_string(),
                                            "1m0s".to_string(),
                                            false);

  // todo: register svc method
  svc_serve.register_fn("say_hello".into(), say_hello);

  // todo: run
  println!("===run here 1 ===");
  svc_serve.run().await;

  println!("===run here 2 ===");

  Ok(())
}

#[actix_web::main]          // 这是一个注解, 类似java的@
async fn main_web() -> std::io::Result<()> {
  // todo: 这一句代表是从env环境变量中读取， 如果没有则设为info
  env_logger::from_env(Env::default().default_filter_or("info")).init();

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





