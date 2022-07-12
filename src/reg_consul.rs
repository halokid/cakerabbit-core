use crate::reg::{check_service, RegisterImpl};
use crate::CakeError;
use consul_rs_plus::Client;
use std::{thread, time};
use consul_rs_plus::pkg::CustomError;


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
    let val = String::from("typ=cakeRabbit");
    let kv_session = c.session_set("0.001s".to_string(),
                                   "".to_string(),
                                   "".to_string(),
                                   "delete".to_string(),
                                   (&self.svc_ttl).to_string());
    trace!("kv_session -->>> {}", kv_session);
    thread::sleep(time::Duration::from_secs(1)); // todo: 确保session生成

    let del_kv_ers = c.kv_delete_both_session(&key);
    match del_kv_ers {
      Err(err) => {
        info!("[do_reg] === Not found old service addr register {}", err);
      }
      Ok(ok) => {
        info!("DELETE old service addr register {}", ok);
      }
    }
    thread::sleep(time::Duration::from_secs(1));

    let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
    trace!("write svc addr: {}", ok);
    if !ok {
      return Err(CakeError("svc register addr fail do_reg()".to_string()));
    } else {
      trace!("--- loop write svc info ---");
      loop {
        thread::sleep(time::Duration::from_secs(100));
        let ok = c.session_renew(&kv_session).unwrap();
        debug!("renew session {} {:?}", kv_session, ok);
        let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
        debug!("--- loop register svc -->>> {}", ok);
      }
    }
  }

  fn do_reg_http(&mut self, http_addr: String, typ: &str) -> Result<bool, CakeError> {
    log::info!("http_addr --- {}", http_addr);
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

    let reg_ok = c.kv_set(format!("{}{}", &key_svc, "Http"),
                          format!("{}{}", &self.svc_name.to_string(), "Http"));
    match reg_ok {
      Err(e) => {
        error!("Service {} register error: {}", &self.svc_name, e);
      }
      _ => {}
    }

    let http_addr_sp: Vec<&str> = http_addr.split(":").collect();
    let svc_addr = local_ipaddress::get().unwrap();
    let key = format!("{}{}/http@{}:{}", path_prex,
                      format!("{}{}", &self.svc_name, "Http"),
                      &svc_addr, http_addr_sp[1]);
    // let val = String::from("typ=rust");
    let val = format!("typ={}", typ);
    let kv_session = c.session_set("0.001s".to_string(),
                                   "".to_string(),
                                   "".to_string(),
                                   "delete".to_string(),
                                   (&self.svc_ttl).to_string());
    trace!("kv_session -->>> {}", kv_session);
    thread::sleep(time::Duration::from_secs(1)); // todo: 确保session生成

    let del_kv_ers = c.kv_delete_both_session(&key);
    match del_kv_ers {
      Err(err) => {
        info!("[do_reg_http] === Not found old service addr register {}", err);
      }
      Ok(ok) => {
        info!("DELETE old service addr register {}", ok);
      }
    }
    thread::sleep(time::Duration::from_secs(1));

    let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
    trace!("write svc addr: {}", ok);
    if !ok {
      return Err(CakeError("svc register addr fail do_reg_http()".to_string()));
    } else {
      trace!("--- loop write svc info ---");
      loop {
        thread::sleep(time::Duration::from_secs(100));
        let ok = c.session_renew(&kv_session).unwrap();
        debug!("renew session {} {:?}", kv_session, ok);
        let val_read = c.kv_get(&key);
        debug!("read the exist key val -->>> {:?}", val_read.as_str());
        // let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
        let ok = c.kv_set_with_session(&key.to_string(), &val_read, &kv_session.to_string()).unwrap();
        debug!("--- loop register svc -->>> {}", ok);
      }
    }
  }

  fn do_reg_external(&mut self, svc_address: String, typ: &str,
    interval: u64) -> Result<bool, CakeError> {
    log::info!("svc_addr --- {}", svc_address);
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

    // 先检查svc的key是否存在
    let svc_reg = c.kv_get(format!("{}", &key_svc));
    if svc_reg.eq("keyNoExists_or_valIsNull") {
      let reg_ok = c.kv_set(format!("{}", &key_svc),
                            format!("{}", &self.svc_name.to_string()));
      match reg_ok {
        Err(e) => {
          error!("Service {} register error: {}", &self.svc_name, e);
        }
        _ => {}
      }
    }

    // let http_addr_sp: Vec<&str> = http_addr.split(":").collect();
    // let svc_addr = local_ipaddress::get().unwrap();
    let key = format!("{}{}/http@{}", path_prex,
                      &self.svc_name,
                      svc_address);
    log::info!("do_reg_external key -->>> {}", key);

    // todo: if exists the node service key, it means some thread is processing the register
    // todo: just return the fn, but this is a bug!!! if the service stop at some time, but the
    // todo: service info is still in memory, so when the status-checker check the service status
    // todo: is false, then the process will stop loop register the service, but at this time
    // todo: the service start again!!! then service send a register request to process, the
    // todo: process think the service already exist, it will not register again! then the
    // todo: start-again service register fail!!!! solution is when the service check status
    // todo: false, then remove the service info from the memory!
    // todo: for the comunicate stable for the service, if the status-checker is false, we should
    // todo: remove the service info in Consul at once??
    let service_node_val = c.kv_get(&key);

    // todo: if the service info the register-center, dont do anything???
    /*
    if !service_node_val.eq("keyNoExists_or_valIsNull") {
      log::warn!("the service node key: {}, val: {} exists, exit Thread {:?}", &key,
        service_node_val, thread::current().id());
      // todo: exit the Tread
      return Ok(true);
    }
     */

    // let val = String::from("typ=rust");
    let val = format!("typ={}", typ);
    let kv_session = c.session_set("0.001s".to_string(),
                                   "".to_string(),
                                   "".to_string(),
                                   "delete".to_string(),
                                   (&self.svc_ttl).to_string());
    info!("kv_session -->>> {}", kv_session);
    thread::sleep(time::Duration::from_secs(1)); // todo: 确保session生成

    let del_kv_ers = c.kv_delete_both_session(&key);
    match del_kv_ers {
      Err(err) => {
        info!("[do_reg_http] === Not found old service addr register {}", err);
      }
      Ok(ok) => {
        info!("DELETE old service addr register {}", ok);
      }
    }
    thread::sleep(time::Duration::from_secs(1));

    let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
    info!("write svc addr: {}", ok);
    if !ok {
      return Err(CakeError("svc register addr fail do_reg_external()".to_string()));
    } else {
      trace!("--- loop write svc info ---");
      loop {
        thread::sleep(time::Duration::from_secs(interval));
        if !check_service(&self.svc_name, svc_address.as_str()) {
          // todo: service checker is false!!!
          break;
        }
        // let ok = c.session_renew(&kv_session).unwrap();
        // debug!("renew session {} {:?}", kv_session, ok);

        let sess_renew_res = c.session_renew(&kv_session);
        match sess_renew_res {
          Err(e) => {
            debug!("[ERROR] -->>> renew session {} {:?} in {:?}", kv_session, ok,
              thread::current().id());
          }
          _ => {}
        }

        let ok = c.kv_set_with_session(&key.to_string(), &val.to_string(), &kv_session.to_string()).unwrap();
        info!("--- loop register svc --- {}, {}, in {:?}", &key, ok, thread::current().id());
      }

      // Ok(true)
      Err(CakeError(format!("[do_reg_external] --- service: {}, node: {} status is false", &self.svc_name ,svc_address)))
      // todo: remove the service info from memory
    }
  }

  fn do_reg_external_nosession(&mut self, svc_address: String, typ: &str,
                     interval: u64) -> Result<bool, CakeError> {
    log::info!("svc_addr --- {}", svc_address);
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

    // 先检查svc的key是否存在
    let svc_reg = c.kv_get(format!("{}", &key_svc));
    if svc_reg.eq("keyNoExists_or_valIsNull") {
      let reg_ok = c.kv_set(format!("{}", &key_svc),
                            format!("{}", &self.svc_name.to_string()));
      match reg_ok {
        Err(e) => {
          error!("Service {} register error: {}", &self.svc_name, e);
        }
        _ => {}
      }
    }

    let key = format!("{}{}/http@{}", path_prex,
                      &self.svc_name,
                      svc_address);
    log::info!("do_reg_external key -->>> {}", key);
    let service_node_val = c.kv_get(&key);

    // let val = String::from("typ=rust");
    let val = format!("typ={}", typ);

    let ok = c.kv_set(&key.to_string(), &val.to_string()).unwrap();
    info!("register no session service -->>> {}, res -->>> {}", &key, ok);
    if !ok {
      return Err(CakeError("svc register addr fail do_reg_external_nosession()".to_string()));
    } else {
      trace!("--- loop write svc info ---");
      loop {
        thread::sleep(time::Duration::from_secs(interval));
        if !check_service(&self.svc_name, svc_address.as_str()) {
          // todo: service checker is false!!! delete the service info and break
          let res = c.kv_delete(&key.to_string());
          match res {
            Ok(_) => {
              log::info!("-->>> service {:?} status not OK, delete info success", &key)
            }
            Err(_) => {
              log::error!("-->>> service {:?} status not OK, delete info fail!!", &key)
            }
          }
          break;
        }
        // todo: if the service status ok, do nothing.
        info!("service {} status OK in Tread {:?}", &key, thread::current().id());
      }

      // Ok(true)
      Err(CakeError(format!("[do_reg_external_nosession] --- service: {}, node: {} status is false", &self.svc_name ,svc_address)))
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
  trace!("svc_nodes -->>> {:?}", svc_nodes);
}



