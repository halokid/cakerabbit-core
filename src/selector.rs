
use crate::CakeResult;
use rand::{ distributions::uniform, Rng };
use rand::distributions::Uniform;
use serde::{Serialize, Deserialize};

pub trait Selector {
  fn select(&mut self) -> String;
  fn update_serv_nodes(&mut self, serv_new_nodes: Vec<String>) -> CakeResult<bool>;
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SelectorTyp {
  Random,
  RoundRobin,
}

pub fn new_selector(st: &SelectorTyp, serv_nodes: Vec<String>) -> Box<dyn Selector> {
  return match st {
    SelectorTyp::RoundRobin => {
      trace!("--- choose RoundRobinSelect ---");
      Box::new(RoundRobinSelect::new(serv_nodes))
    },
    SelectorTyp::Random => {
      trace!("--- choose RandomSelect ---");
      Box::new(RandomSelect::new(serv_nodes))
    }
  }
}

// -----------------------------------------------------------------------

struct RandomSelect {
  pub serv_nodes:   Vec<String>,
}

impl RandomSelect {
  fn new(serv_nodes: Vec<String>) -> Self {
    RandomSelect{ serv_nodes }
  }
}

impl Selector for RandomSelect {
  fn select(&mut self) -> String {

    let serv_nodes_len = self.serv_nodes.len();
    if serv_nodes_len == 0 {
      return "".to_string();
    }
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, serv_nodes_len);
    let random_num = rng.sample(&range);
    let node = self.serv_nodes.get(random_num).unwrap();
    node.to_string()
  }

  fn update_serv_nodes(&mut self, serv_new_nodes: Vec<String>) -> CakeResult<bool> {
    self.serv_nodes = serv_new_nodes;
    Ok(true)
  }
}

// -----------------------------------------------------------------------

struct RoundRobinSelect {
  serv_nodes:   Vec<String>,
  round_index:  usize,
}

impl RoundRobinSelect {
  fn new(serv_nodes: Vec<String>) -> Self {
    RoundRobinSelect{ serv_nodes, round_index: 0 }
  }
}

impl Selector for RoundRobinSelect {
  fn select(&mut self) -> String {
    let serv_nodes_len = self.serv_nodes.len();
    if serv_nodes_len == 0 {
      return "".to_string();
    }

    let mut index = self.round_index;
    index = index % serv_nodes_len;
    trace!("index -->>> {}", index);
    let node = self.serv_nodes.get(index).unwrap();
    self.round_index = index + 1;
    node.to_string()
  }

  fn update_serv_nodes(&mut self, serv_new_nodes: Vec<String>) -> CakeResult<bool> {
    self.serv_nodes = serv_new_nodes;
    Ok(true)
  }
}


mod tests {
  #![feature(impl_trait_in_bindings)]
  use rand::{ distributions::uniform, Rng };
  use rand::distributions::Uniform;
  use super::*;
  use std::time;
  use std::thread;

  #[test]
  fn test_random_selector() {
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 5);
    let x = rng.sample(&range);
    trace!("random i32 -->>> {}", x);

    let mut selector = new_selector(&SelectorTyp::Random,
                                    vec!["8.8.8.8:90".to_string(),
                                         "7.7.7.7:99".to_string(), "6.6.6.6:88".to_string()]);
    let node = selector.select();
    trace!("node -->>> {}", node);

    let new_nodes = vec!["1.1.1.1:33".to_string(), "2.2.2.2:55".to_string()];
    let ok = selector.update_serv_nodes(new_nodes);
    trace!("ok -->>> {:?}", ok);
    let node1 = selector.select();
    trace!("node1 -->>> {}", node1);
  }

  #[test]
  fn test_roundrobin_selector() {
    let mut selector = new_selector(&SelectorTyp::RoundRobin,
                                    vec!["8.8.8.8:90".to_string(),
                                         "7.7.7.7:99".to_string(), "6.6.6.6:88".to_string()]);
    let node1 = selector.select();
    trace!("node1 -->>> {}", node1);

    thread::sleep(time::Duration::from_secs(1));
    let node2 = selector.select();
    trace!("node2 -->>> {}", node2);

    thread::sleep(time::Duration::from_secs(1));
    let node3 = selector.select();
    trace!("node3 -->>> {}", node3);

    thread::sleep(time::Duration::from_secs(1));
    let node4 = selector.select();
    trace!("node4 -->>> {}", node4);
  }
}




