
use std::{io, time};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use cakerabbit_core::{Client, Value, Register, new_selector, SelectorTyp, CakeClient, CakeError, FailMode, Failtry};
use tokio_util::compat::TokioAsyncReadCompatExt;
use std::future::Future;
use futures::TryFutureExt;
use std::thread;

#[tokio::main]
async fn main() -> io::Result<()> {
  env_logger::init();

  let fm = FailMode::Failtry(Failtry{ retries: 3 });
  let mut cake_client = CakeClient::new("cake/".into(),
                                        "EchoRs".into(),
                                        "consul".into(),
                                        "localhost:8500".into(),
                                        SelectorTyp::RoundRobin,
                                        FailMode::FailFast);
                                         // fm);
  let res = cake_client.call("say_hello", &["foo".into()]).await;
  match res {
    Ok(rsp) => {
      println!("rsp ------------ {}", rsp);
    }

    Err(err) => {
      println!("err ------------------ {:?}", err);
    }
  }

  thread::sleep(time::Duration::from_secs(2));
  // todo: cake_clientx borrow to anthor thread, because use tokio async fn here!
  match cake_client.call("say_hello", &["foo2".into()]).await {
    Ok(rsp) => {
      println!("rsp2 ------------ {}", rsp);
    },
    Err(err) => {
      println!("err2 ------------------ {:?}", err);
    }
  }

  Ok(())
}


