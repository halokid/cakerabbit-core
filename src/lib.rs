
#![feature(test)]

#[macro_use]
extern crate log;

mod codec;
mod endpoint;
mod errors;
mod message;
mod cakeservice;
mod reg;
mod reg_consul;
mod selector;
mod node;
mod cakeclient;
mod failmode;

pub use crate::endpoint::{
  serve, Ack, Client, Endpoint, Response,
};

pub use crate::cakeservice::{
  Service, ServiceWithClient, CakeServiceServe, CakeFn, CakeResult
};

pub use crate::reg::{
  Register
};

pub use crate::selector::{
  Selector, new_selector, SelectorTyp
};

pub use crate::failmode::{
  FailMode, Failover, Failtry,
};

pub use crate::errors::{
  CakeError,
};

pub use crate::cakeclient::{
  CakeClient,
};

pub use rmpv::{Integer, Utf8String, Value};

// todo: global extern test module
extern crate test;

pub fn add_two(a: i32) -> i32 {
  a + 2
}

#[cfg(test)]
mod tests {
  use super::*;
  use test::Bencher;

  #[test]
  fn test_add_two() {
    assert_eq!(4, add_two(2));
  }

  #[bench]
  fn bench_add_two(b: &mut Bencher) {
    b.iter(| | add_two(2));
  }
}


