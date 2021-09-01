
use crate::{SelectorTyp, Response, Client, Register, new_selector, CakeError, CakeResult,
            Failover, Failtry};
use rmpv::Value;
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;
use std::io;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use crate::failmode::FailMode;
use crate::errors::cake_errors;

#[derive(Clone, Serialize, Deserialize)]
pub struct CakeClient {
  svc_prefix:       String,
  svc_name:         String,
  reg_typ:          String,
  reg_addr:         String,
  selector_typ:     SelectorTyp,
  failmode:         FailMode,
}


impl CakeClient {
  pub fn new(svc_prefix: String, svc_name: String, reg_typ: String, reg_addr: String,
             selector_typ: SelectorTyp, failmode: FailMode) -> Self {
    CakeClient {
      svc_prefix,
      svc_name,
      reg_typ,
      reg_addr,
      selector_typ,
      failmode,
    }
  }

  pub async fn call(&mut self, method: &str, params: &[Value]) -> CakeResult<String> {
    match &self.failmode {
      FailMode::FailFast => {
        let res = self.wrap_call(method, params).await;
        return res;
      },

      FailMode::Failtry(fm ) => {
        let mut retries = fm.retries;
        while retries >= 0 {
          let res = self.wrap_call(method, params).await;
          match res {
            Ok(rsp) => {
              return Ok(rsp.to_string())
            },

            Err(err) => {
              error!("cakeClient call Error --------------- {:?}", err)
            }
          }
          retries -= 1;
        }

        // return Ok("error".to_string());
        return Err(CakeError(cake_errors("callFailtryErr")));
      },

      _ => {}
    }

    return Ok("".to_string());
  }

  pub async fn wrap_call(&mut self, method: &str, params: &[Value]) -> CakeResult<String> {
    let svc_namex = &self.svc_name.clone();
    let mut reg = Register::new_for_client((&self.reg_typ).to_string(),
                                           (&self.reg_addr).to_string(),
                                           svc_namex.to_string(),
                                           (&self.svc_prefix).to_string(), false);
    let svc_nodes = reg.get_service_nodes(self.svc_name.clone());
    trace!("svc_nodes ------------- {:?}", svc_nodes);
    let mut selector = new_selector(&self.selector_typ,
                                    svc_nodes);
    let node_addr = selector.select();

    let socket = TcpStream::connect(&node_addr).await?;
    let inner_client = Client::new(socket.compat());
    let rsp = inner_client.call(method, params);

    match rsp.await {
      Ok(rspx) => {
        Ok(rspx.to_string())
     }

      Err(err) => {
        Err(CakeError(format!("client.call Error: {:?}", err)))
      }
    }

  }
}

