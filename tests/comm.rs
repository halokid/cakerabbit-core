
use serde_json::Error;
use serde::{Deserialize, Serialize};

#[test]
fn test_comm() {
  trace!("test comm");
}

#[derive(Debug, Serialize, Deserialize)]
struct Animal {
  name:   String,
}

#[test]
fn test_struct_to_vec() {
  let cat = Animal{
    name: "cat".to_string()
  };
  trace!("cat -------- {:?}", cat);

  // struct to vec
  let stu = serde_json::to_vec(&cat);
  match stu {
    Ok(stu) => {
      trace!("stu -------- {:?}", stu);
      // vec to struct
      let catx = serde_json::from_slice::<Animal>(&stu);
      trace!("catx ------------ {:?}", catx);
    }
    Err(err) => { trace!("err ------------- {:?}", err); }
  }
}


