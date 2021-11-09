communicate trace for client call service
=====================

```shell

[2021-11-09T09:43:47Z TRACE cakerabbit_core::cakeclient] svc_nodes ------------- ["8.8.8.8:9527"]
[2021-11-09T09:43:47Z TRACE cakerabbit_core::selector] --- choose RoundRobinSelect ---
[2021-11-09T09:43:47Z TRACE cakerabbit_core::selector] index --------- 0
[2021-11-09T09:43:47Z TRACE mio::poll] registering event source with poller: token=Token(0), interests=READABLE | WRITABLE
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] InnerEndpoint: polling
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] New call (method=say_hello, params=[String(Utf8String { s: Ok("foo") })])
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] === InnerEndpoint handler.send_outgoing ===
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Response: polling
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] === impl MessageHandler for InnerClient invoke send_outgoing ===
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Response: polling
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Polling client requests channel
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Got request from client: Request { id: 0, method: "say_hello", params: [String(Utf8String { s: Ok("foo") })] }
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] No new request from client
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Polling client notifications channel
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] No new notification from client
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] flushing framed transport
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] writing; remaining=18
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] framed transport flushed
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] === Polling stream ===
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Transport: polling
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] notifying the reactor that we\'re not done yet


[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] InnerEndpoint: polling
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] === InnerEndpoint handler.send_outgoing ===
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] === impl MessageHandler for InnerClient invoke send_outgoing ===
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Polling client requests channel
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] No new request from client
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Polling client notifications channel
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] No new notification from client
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] flushing framed transport
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] framed transport flushed
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] === Polling stream ===
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Transport: polling
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] attempting to decode a frame
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] frame decoded from buffer
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] ---check msg struct---
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] ---handle_incoming msg---.
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Received Response(Response { id: 1, result: Ok(String(Utf8String { s: Ok("{\"name\":\"foo\"}") })) })
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Forwarding response to the client.
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Response: polling
rsp ------------ "{\"name\":\"foo\"}"
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] Transport: polling
[2021-11-09T09:43:47Z TRACE tokio_util::codec::framed_impl] attempting to decode a frame
[2021-11-09T09:43:47Z TRACE cakerabbit_core::endpoint] notifying the reactor that we\'re not done yet
[2021-11-09T09:43:47Z TRACE mio::poll] deregistering event source from poller

```