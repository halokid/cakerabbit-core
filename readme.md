CakeRabbit-core
==================================

A rust microservice framework, this is the core kernel for the project.
- Transport
    - [x] TCP 
    - [x] support tikio async future
    - [x] msgpack codec transport 
    - [ ] grpc codec transport

- Service 
    - [x] register service fn 

- Server 
    - [x] RPC server(tokio future)

- Cilent 
    - [x] RPC client(tokio future)

- Register 
    - [x] Consul 

- Failmode 
    - [ ] Failover 
    - [ ] Failfast
    - [ ] Failtry

- Selector 
    - [x] Random
    - [x] RoundRobin



## Usage 
set in Cargo.toml:

```toml
[dependencies]
cakerabbit-core = "0.1.0"
```



## Example

code a EchoRs service, register method say_hello().
if you want to devlop use local project source, check the example in folder 
./examples/echo_use_localsource.
run command, check the toml:
```shell
cargo.exe run --example echoserver

cargo.exe run --example echoclient
```

if you want to devlop use crate.io source,  check the example in folder
./examples/echo_use_crateio
run command, check the toml:
```shell
cd ./examples/echo_use_crateio

cargo.exe run --example echoserver

cargo.exe run --example echoclient
```


### Server

```rust
const SVC_NAME: &str = "EchoRs";

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
struct SayHelloReply {
  name: String,
}

fn say_hello(params: &[Value]) -> CakeResult<Vec<u8>> {
  if let Value::String(ref value) = params[0] {
    if let Some(val) = value.as_str() {
      let rsp = SayHelloReply {
        name: "foo".to_string()
      };
      let rsp_vec = serde_json::to_vec(&rsp).unwrap();
      return Ok(rsp_vec);
    }
  }
  Ok(Vec::from("wrong param".to_string()))
}

#[tokio::main]
async fn main() -> io::Result<()> {
  env_logger::from_env(Env::default().default_filter_or("info")).init();
  let mut svc_serve = CakeServiceServe::new(SVC_NAME.to_string(),
                                            "cake/".to_string(),
                                            "0.0.0.0:9527".to_string(),
                                            "consul".to_string(),
                                            "localhost:8500".to_string(),
                                            "1m0s".to_string(),
                                            false);
  // todo: register svc method
  svc_serve.register_fn("say_hello".into(),
                        say_hello,
                        &["foo".into()]);

  // todo: run
  svc_serve.run().await;

  Ok(())
}


```

### client

code a client call EchoRs service

```rust
#[tokio::main]
async fn main() -> io::Result<()> {
  env_logger::init();

  let fm = FailMode::Failtry(Failtry{ retries: 3 });
  let mut cake_client = CakeClient::new("cake/".into(),
                                        "EchoRs".into(),
                                        "consul".into(),
                                        "localhost:8500".into(),
                                        SelectorTyp::RoundRobin,
                                        FailMode::FailFast);
                                         // fm);
  let res = cake_client.call("say_hello", &["foo".into()]).await;
  match res {
    Ok(rsp) => {
      println!("rsp ------------ {}", rsp);
    }
    Err(err) => {
      println!("err ------------------ {:?}", err);
    }
  }
}
```

## Build a service use rpcx-plus-gateway
you can build a service use 


## Features
1. Easy use, keep in simple

2. Gateway support to external service use RESTFUL api

3. More program language support,   cakeRabbit-code transport codec base on msgpack(grpc etc.), it means if you write a service use rust, it can be call by other language.

4. UI for service governance,  service register center can use consul, etcd, zookeeper.

   

## Performance













