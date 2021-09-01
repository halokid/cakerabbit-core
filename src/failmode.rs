
use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub enum FailMode {
  Failover(Failover),
  FailFast,
  Failtry(Failtry),
  FailBackup
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Failover {
  pub retries:      i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Failtry {
  pub retries:      i32,
}






