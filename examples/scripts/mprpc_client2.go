package main

import (
  "fmt"
  "github.com/msgpack-rpc/msgpack-rpc-go/rpc"
  "log"
  "net"
  "reflect"
)

type Resolver map[string]reflect.Value

func (self Resolver) Resolve(name string, arguments []reflect.Value) (reflect.Value, error) {
  return self[name], nil
}

func echo(test string) (string, fmt.Stringer) {
  return "Hello, " + test, nil
}

func add(a, b uint) (uint, fmt.Stringer) {
  return a + b, nil
}

func main() {
  //conn, err := net.Dial("tcp", "127.0.0.1:9527")
  conn, err := net.Dial("tcp", "127.0.0.1:10011")
  if err != nil {
    fmt.Println("fail to connect to server.")
    return
  }
  client := rpc.NewSession(conn, true)

  //retval, xerr := client.Send("say_hello", "foo")
  log.Println("--- run here 1 ---")
  retval, xerr := client.Send("company", "2")
  //retval, xerr := client.Send("company", "2", "3", "i am keyword")
  log.Println("--- run here 2 ---")
  if xerr != nil {
    fmt.Println("error ---------", xerr)
    return
  }
  fmt.Println(retval.String())
  fmt.Println("done")
  //fmt.Println(rpc.CoerceInt(retval))
}

func comm() {
  res := Resolver{"echo": reflect.ValueOf(echo), "add": reflect.ValueOf(add)}
  serv := rpc.NewServer(res, true, nil)
  l, err := net.Listen("tcp", "127.0.0.1:50000")
  serv.Listen(l)
  go (func() { serv.Run() })()

  conn, err := net.Dial("tcp", "127.0.0.1:50000")
  if err != nil {
    fmt.Println("fail to connect to server.")
    return
  }
  client := rpc.NewSession(conn, true)

  retval, xerr := client.Send("echo", "world")
  if xerr != nil {
    fmt.Println(xerr)
    return
  }
  fmt.Println(retval.String())

  retval, xerr = client.Send("add", 2, 3)
  if xerr != nil {
    fmt.Println(xerr)
    return
  }
  fmt.Println(rpc.CoerceInt(retval))
}


