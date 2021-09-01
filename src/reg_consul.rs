use crate::reg::RegisterImpl;
use crate::CakeError;
use consul_rs_plus::Client;
use std::{thread, time};


pub struct RegConsul {
  regaddr: String,
  svc_name: String,
  svc_prefix: String,
  svc_port: String,
  svc_ttl: String,
  debug: bool,
}

impl RegConsul {
  pub fn new(regaddr: String, svc_name: String, svc_prefix: String,
             svc_port: String, svc_ttl: String, debug: bool) -> Self {
    RegConsul { regaddr, svc_name, svc_prefix, svc_port, svc_ttl, debug }
  }
}

impl RegisterImpl for RegConsul {
  fn do_reg(&mut self) -> Result<bool, CakeError> {
    let mut path_prex = &self.svc_prefix;
    let mut regaddr_iter = &self.regaddr.split(":");
    let regaddr_vec: Vec<&str> = regaddr_iter.clone().collect();
    let consul_host = regaddr_vec[0];
    let consul_port = regaddr_vec[1];
    let consul_port_u16 = consul_port.parse::<u16>().unwrap();

    let mut c = Client::new(consul_host, consul_port_u16);
    c.debug = self.debug;
    // set key
    let key_svc = format!("{}{}", path_prex, &self.svc_name);

    let reg_ok = c.kv_set(&key_svc, &self.svc_name.to_string());
    match reg_ok {
      Err(e) => {
        error!("Service {} register error: {}", &self.svc_name, e);
      }
      _ => {}
    }

    let svc_addr = local_ipaddress::get().unwrap();
    let key = format!("{}{}/tcp@{}:{}", path_prex, &self.svc_name, &svc_addr, &self.svc_port);
    let val = String::from("typ=rust");
    let kv_session = c.session_set("0.001s".to_string(),
                                   "".to_string(),
                                   "".to_string(),
                                   "delete".to_string(),
                                   (&self.svc_ttl).to_string());
    trace!("kv_session --------- {}", kv_session);
    thread::sleep(time::Duration::from_secs(1)); // todo: 确保session生成

    let del_kv_ers = c.kv_delete_both_session(&key);
    match del_kv_ers {
      Err(err) => {
        info!("Not found old service addr register {}", err);
      }
      Ok(ok) => {
        info!("DELETE old service addr register {}", ok);
      }
    }
    thread::sleep(time::Duration::from_secs(1));

    let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
    trace!("write svc addr: {}", ok);
    if !ok {
      return Err(CakeError("svc register addr fail".to_string()));
    } else {
      trace!("--- loop write svc info ---");
      loop {
        thread::sleep(time::Duration::from_secs(100));
        let ok = c.session_renew(&kv_session).unwrap();
        debug!("renew session {} {:?}", kv_session, ok);
        let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
        debug!("--- loop register svc --- {}", ok);
      }
    }
  }

  fn watch_services(&mut self) -> Result<bool, CakeError> {
    todo!()
  }

  fn get_service_nodes(&self, service: String) -> Vec<String> {
    let mut regaddr_iter = &self.regaddr.split(":");
    let regaddr_vec: Vec<&str> = regaddr_iter.clone().collect();
    let consul_host = regaddr_vec[0];
    let consul_port = regaddr_vec[1];
    let consul_port_u16 = consul_port.parse::<u16>().unwrap();

    let mut c = Client::new(consul_host, consul_port_u16);
    c.debug = self.debug;
    let svc_nodes = c.kv_get_folder(format!("{}/{}", &self.svc_prefix,
                                            service)).unwrap();
    // svc_nodes
    let mut nodes = Vec::new();
    let svc_prefix = &self.svc_prefix;
    let svc_name = &self.svc_name;
    let repl = format!("{}{}/tcp@", svc_prefix, svc_name);
    for node in svc_nodes {
      // let repl = format!("{}{}/tcp@", svc_prefix, svc_name);
      let node_fix = node.replace(&repl, "");
      nodes.push(node_fix);
    }
    nodes
  }
}

#[test]
fn test_get_service_nodes() {
  let reg_consul = RegConsul::new("8.8.8.8:8500".to_string(), "Echo".to_string(), "cake".to_string(), "88".to_string(), "10m0s".to_string(), true);
  let svc_nodes = reg_consul.get_service_nodes("Echo".to_string());
  trace!("svc_nodes ------ {:?}", svc_nodes);
}



