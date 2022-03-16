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

const SVC_NAME: &str = "EchoRs";

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
struct SayHelloReply {
  name: String,
}

fn say_hello(params: &[Value]) -> CakeResult<Vec<u8>> {
  println!("say_hello params -->>> {:?}", params);
  let bigdata = get_big_data();
  Ok(Vec::from(bigdata))
}

fn get_big_data() -> String {
  let s = "hello-";
  let mut sx = "".to_string();
  for _ in 0..1000000 {
    sx = format!("{}{}", sx, s);
  }
  sx.to_string()
}

#[tokio::main]
async fn main() -> io::Result<()> {
  // let sx = get_big_data();
  // println!("sx --- {}", sx);
  // std::process::exit(0);

  env_logger::from_env(Env::default().default_filter_or("info")).init();

  let mut svc_serve = CakeServiceServe::new(SVC_NAME.to_string(),
                                            "pomid/".to_string(),
                                            "0.0.0.0:9527".to_string(),
                                            "consul".to_string(),
                                            "consul_test:8500".to_string(),
                                            "1m0s".to_string(),
                                            false,
                                            "");

  // todo: register svc method
  svc_serve.register_fn("say_hello".into(), say_hello);

  // todo: run
  svc_serve.run().await;

  Ok(())
}


