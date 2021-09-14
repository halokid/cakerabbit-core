#![feature(test)]
extern crate test;

use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use cakerabbit_core::{Client, Value};
use tokio_util::compat::TokioAsyncReadCompatExt;

#[tokio::main]
async fn client_to_serv() -> io::Result<()> {
  let addr: SocketAddr = "127.0.0.1:9527".parse().unwrap();
  let socket = TcpStream::connect(&addr).await?;
  let client = Client::new(socket.compat());

  client.notify("I am notify message", &[]);

  match client.request("say_hello", &["foo".into()]).await {
    Ok(response) => println!("Response: {:?}", response),
    Err(e) => println!("Error: {:?}", e),
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use test::Bencher;
  use crate::client_to_serv;


  #[bench]
  fn bench_client_to_serv(b: &mut Bencher) {
    // b.iter(|| println!("bench..."));
    b.iter(|| client_to_serv() );
  }
}


