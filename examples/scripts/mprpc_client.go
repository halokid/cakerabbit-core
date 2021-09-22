
package main

import (
  "github.com/halokid/ColorfulRabbit"
  "github.com/vmihailenco/msgpack"
  "log"
  "net"
  "time"
)

type Request struct {
  //id          int32
  method      string
  ARGS        []string
  //params      *[]interface{}
  //params      []uint8
  //params      []string
  //params      []byte
}

func main() {
  var conn net.Conn
  var err error
  conn, err = net.DialTimeout("tcp", "127.0.0.1:9527", time.Duration(time.Second * 5))
  ColorfulRabbit.CheckError(err, "mprpc go connection error")
  defer conn.Close()

  if tc, ok := conn.(*net.TCPConn); ok {
    tc.SetKeepAlive(true)
    tc.SetKeepAlivePeriod(3 * time.Minute)
  }
  // write
  //request :=`
  //          {
  //            "id": 0,
  //            "method": "say_hello",
  //            "params": ["foo"]
  //          }
  //          `
  //_, err = conn.Write([]byte(request))

  ///*
  request := Request{
    //id:     0,
    method: "say_hello",
    ARGS:   []string{},
    //method: "say_hello",
    //params: &[]interface{}{},
    //params: []interface{}{},
    //params: []uint8{},
    //params: []string{},
    //params: []byte{},
  }
  //*/

  //xx :=  []string{"say_hello", "bb"}

  bReq, err := msgpack.Marshal(&request)
  //bReq, err := msgpack.Marshal(&xx)
  ColorfulRabbit.CheckError(err, "mprpc marshal error")
  _, err = conn.Write(bReq)
  ColorfulRabbit.CheckError(err, "mprpc go conn send data error")

  // read
  buf := make([]byte, 1024)
  readLen, err := conn.Read(buf)
  ColorfulRabbit.CheckError(err, "mprpc go read error")
  rsp := string(buf[:readLen - 1])
  log.Printf("rsp -------------- %+v", rsp)
}




