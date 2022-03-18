use std::net::{Shutdown, TcpStream};
use crate::CakeError;
use crate::reg_consul::{RegConsul};
use env_logger::Env;
use serde::__private::de::Content::{U32, U64};
use crate::{CONFIG};
use std::thread;
use std::time;

pub trait RegisterImpl {

  fn do_reg(&mut self) -> Result<bool, CakeError>;

  fn do_reg_http(&mut self, http_addr: String, typ: &str) -> Result<bool, CakeError>;

  fn do_reg_external(&mut self, svc_address: String, typ: &str,
    interval: u64) -> Result<bool, CakeError>;

  fn watch_services(&mut self) -> Result<bool, CakeError>;

  fn get_service_nodes(&self, service: String) -> Vec<String>;
}

pub struct Register {
  adapter: String,
  debug:   bool,
}

impl Register {
  pub fn new_for_service(adapter: String, regaddr: String, svc_name: String, svc_prefix: String, svc_port: String, svc_ttl: String, debug: bool) -> Box<dyn RegisterImpl> {
    return match adapter.as_str() {
      "consul" => {
        Box::new(RegConsul::new(regaddr, svc_name, svc_prefix,
                                svc_port, svc_ttl, debug))
      }

      _ => {
        Box::new(RegConsul::new(regaddr, svc_name, svc_prefix,
                                svc_port, svc_ttl, debug))
      }
    }
  }

  pub fn new_for_client(adapter: String, regaddr: String, svc_name: String,
                        svc_prefix: String, debug: bool) -> Box<dyn RegisterImpl> {
    return match adapter.as_str() {
      "consul" => {
        Box::new(RegConsul::new(regaddr, svc_name, svc_prefix,
                                "".into(), "".into(), debug))
      }

      _ => {
        Box::new(RegConsul::new(regaddr, svc_name, svc_prefix,
                                "".into(), "".into(), debug))
      }
    }
  }
}

// check the status for external service, use tcp connect port
pub fn check_service(svc_name: &str, svc_address: &str) -> bool {
  // log::info!("=== check_service {} ===", svc_address);
  let mut stream = TcpStream::connect(svc_address);
  match stream {
    Ok(s) => {
      log::info!("=== check_service {}, {} === service status: true", svc_name, svc_address);
      s.shutdown(Shutdown::Both).unwrap();
      return true;
    }

    Err(_) => {
      log::error!("=== check_service {}, {} === service status: false,\
       checker will retries {} times in ThreadID {:?}", svc_name, svc_address, CONFIG["service_check_retries"], thread::current().id());

      let service_check_retries = CONFIG["service_check_retries"].parse::<u64>().unwrap();
      for i in 0..service_check_retries {
        log::error!("=== check_service {}, {} === service status: false, checker retries \
        times {} in ThreadID {:?}", svc_name, svc_address, i, thread::current().id());
        let check_res = inner_check_service(svc_address);
        if check_res {      // if service status OK again
          return true;
        }
        let check_itval = CONFIG["service_check_interval"].parse::<u64>().unwrap();
        thread::sleep(time::Duration::from_secs(check_itval));
      }

      log::error!("=== check_service {}, {} === service status: false in ThreadID {:?}",
        svc_name, svc_address, thread::current().id());
      return false;
    }
  }
}

// todo: retry check service will occur a problem, if thread-A is retrying check service, when
// todo: the checker until 80 seconds, this time the service restart again, and online status
// todo: within 10 seconds, the API create a thread-B to handle this. and the thread-A next 20
// todo: seconds check will be OK, in this case, that has two threads to handler one service,
// todo: it should not be. so this solution is not good enough!!
// todo: but this solution can run OK, because when new request for register service, it has
// todo: two condition
// todo: 1 is the service info is not in register center, in this case, the new
// todo: request will recreate the session, so it will not two threads handle one service, the old
// todo: thread will occur [ERROR] `CustomError("renew session err` and exist. the new thread will
// todo: take over it.
// todo: 2 is the service info is still in register center, in this case, the old thread checker
// todo: will check the service status, it is true, old thread will lopp handle again, the new
// todo: thread waill exist when it found the service info is `exist`
fn inner_check_service(svc_address: &str) -> bool {
  let mut stream = TcpStream::connect(svc_address);
  match stream {
    Ok(s) => {
      s.shutdown(Shutdown::Both).unwrap();
      true
    }

    Err(_) => false
  }
}


#[test]
fn test_register_svc() {
  env_logger::from_env(Env::default().default_filter_or("info")).init();
  let mut reg = Register::new_for_service("consul".to_string(),
                                          "8.8.8.8:8500".to_string(),
                                          "my_svc".to_string(),
                                          "cake/".to_string(),
                                          "9527".to_string(),
                                          "1m0s".to_string(),
                                          true);
  let res = reg.do_reg(); match res {
    Ok(_) => {}
    Err(_) => {}
  }
}

#[test]
fn check_service_test() {
  env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
  // let check = check_service("svc_name", "127.0.0.1:8500");
  let check = check_service("service_access_demo", "127.0.0.1:8989");
  println!("check -->>> {}", check);
}




