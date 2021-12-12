use std::net::TcpStream;
use crate::CakeError;
use crate::reg_consul::{RegConsul};
use env_logger::Env;

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
pub fn check_service(svc_address: &str) -> bool {
  log::info!("=== check_service {} ===", svc_address);
  let mut stream = TcpStream::connect(svc_address);
  match stream {
    Ok(_) => {
      log::info!("service status: true");
      return true;
    }

    Err(_) => {
      log::info!("service status: false");
      return false;
    }
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
  let check = check_service("127.0.0.1:8500");
  println!("check --- {}", check);
}




