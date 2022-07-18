
use lazy_static::*;
use std::collections::HashMap;

lazy_static! {
  pub static ref CONFIG: HashMap<&'static str, &'static str> = {
    let mut config = HashMap::new();
    // todo: the service register normal duration time is one minute(60 sec),
    // todo: so 5*40 check time will be OK, it more large than the 60, can cover the time
    // todo: duration service stop and remove info from register center, when the service
    // todo: start again, the service info is not in the register center, this make sure the
    // todo: checker process will not consider the service is exist already and exit register.
    config.insert("service_check_retries", "10");
    config.insert("service_check_interval", "30");

    config
  };
}



