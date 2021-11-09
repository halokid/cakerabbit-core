use std::collections::HashMap;
use std::io;

use std::marker::Unpin;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::channel::{mpsc, oneshot};
use futures::io::{AsyncRead, AsyncWrite};
use futures::{ready, Future, FutureExt, Sink, Stream, TryFutureExt};
use rmpv::Value;
use tokio_util::codec::{Decoder, Framed};
use tokio_util::compat::{Compat, FuturesAsyncWriteCompatExt};

use crate::codec::Codec;
use crate::message::Response as MsgPackResponse;
use crate::message::{Message, Notification, Request};
use crate::cakeservice::{ Service, ServiceWithClient };


struct Server<S> {
  service: S,
  pending_responses: mpsc::UnboundedReceiver<(u32, Result<Value, Value>)>,
  response_sender: mpsc::UnboundedSender<(u32, Result<Value, Value>)>,
}

impl<S: ServiceWithClient> Server<S> {
  fn new(service: S) -> Self {
    let (send, recv) = mpsc::unbounded();

    Server {
      service,
      pending_responses: recv,
      response_sender: send,
    }
  }

  fn send_responses<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    mut sink: Pin<&mut Transport<T>>,
  ) -> Poll<io::Result<()>> {
    trace!("Server: flushing responses");
    loop {
      ready!(sink.as_mut().poll_ready(cx)?);
      match Pin::new(&mut self.pending_responses).poll_next(cx) {
        Poll::Ready(Some((id, result))) => {
          let msg = Message::Response(MsgPackResponse { id, result });
          sink.as_mut().start_send(msg).unwrap();
        }
        Poll::Ready(None) => panic!("we store the sender, it can't be dropped"),
        Poll::Pending => return sink.as_mut().poll_flush(cx),
      }
    }
  }

  fn spawn_request_worker<F: Future<Output = Result<Value, Value>> + 'static + Send>(
    &self,
    id: u32,
    f: F,
  ) {
    trace!("spawning a new task");
    trace!("spawning the task on the event loop");
    let send = self.response_sender.clone();
    tokio::task::spawn(f.map(move |result| send.unbounded_send((id, result))));
  }
}

trait MessageHandler {
  fn handle_incoming(&mut self, msg: Message);

  fn send_outgoing<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    sink: Pin<&mut Transport<T>>,
  ) -> Poll<io::Result<()>>;

  fn is_finished(&self) -> bool {
    false
  }
}

type ResponseTx = oneshot::Sender<Result<Value, Value>>;

pub struct Response(oneshot::Receiver<Result<Value, Value>>);

type AckTx = oneshot::Sender<()>;

pub struct Ack(oneshot::Receiver<()>);

// TODO: perhaps make these bounded (for better backpressure)
type RequestTx = mpsc::UnboundedSender<(Request, ResponseTx)>;
type RequestRx = mpsc::UnboundedReceiver<(Request, ResponseTx)>;

type NotificationTx = mpsc::UnboundedSender<(Notification, AckTx)>;
type NotificationRx = mpsc::UnboundedReceiver<(Notification, AckTx)>;

impl Future for Response {
  type Output = Result<Value, Value>;

  // todo: ==step 3==
  fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    trace!("Response: polling");
    Poll::Ready(match ready!(Pin::new(&mut self.0).poll(cx)) {
      Ok(Ok(v)) => Ok(v),
      Ok(Err(v)) => Err(v),
      Err(_) => Err(Value::Nil),
    })
  }
}

impl Future for Ack {
  type Output = Result<(), ()>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    trace!("Ack: polling");
    Pin::new(&mut self.0).poll(cx).map_err(|_| ())
  }
}

struct InnerClient {
  client_closed: bool,
  request_id: u32,
  requests_rx: RequestRx,
  notifications_rx: NotificationRx,
  pending_requests: HashMap<u32, ResponseTx>,
  pending_notifications: Vec<AckTx>,
}

impl InnerClient {
  fn new() -> (Self, Client) {
    let (requests_tx, requests_rx) = mpsc::unbounded();
    let (notifications_tx, notifications_rx) = mpsc::unbounded();

    let client = Client {
      requests_tx,
      notifications_tx
    };

    let inner_client = InnerClient {
      client_closed: false,
      request_id: 0,
      requests_rx,
      notifications_rx,
      pending_requests: HashMap::new(),
      pending_notifications: Vec::new(),
    };

    (inner_client, client)
  }

  fn process_notifications<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    mut stream: Pin<&mut Transport<T>>,
  ) -> io::Result<()> {
    if self.client_closed {
      return Ok(());
    }

    trace!("Polling client notifications channel");

    while let Poll::Ready(()) = stream.as_mut().poll_ready(cx)? {
      match Pin::new(&mut self.notifications_rx).poll_next(cx) {
        Poll::Ready(Some((notification, ack_sender))) => {
          trace!("Got notification from client.");
          stream
            .as_mut()
            .start_send(Message::Notification(notification))?;
          self.pending_notifications.push(ack_sender);
        }
        Poll::Ready(None) => {
          trace!("Client closed the notifications channel.");
          self.client_closed = true;
          break;
        }
        Poll::Pending => {
          trace!("No new notification from client");
          break;
        }
      }
    }
    Ok(())
  }

  fn send_messages<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    mut stream: Pin<&mut Transport<T>>,
  ) -> Poll<io::Result<()>> {
    self.process_requests(cx, stream.as_mut())?;
    self.process_notifications(cx, stream.as_mut())?;

    match stream.poll_flush(cx)? {
      Poll::Ready(()) => {
        self.acknowledge_notifications();
        Poll::Ready(Ok(()))
      }
      Poll::Pending => Poll::Pending,
    }
  }

  fn process_requests<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    mut stream: Pin<&mut Transport<T>>,
  ) -> io::Result<()> {
    // Don't try to process requests after the requests channel was closed, because
    // trying to read from it might cause panics.
    if self.client_closed {
      return Ok(());
    }
    trace!("Polling client requests channel");
    while let Poll::Ready(()) = stream.as_mut().poll_ready(cx)? {
      match Pin::new(&mut self.requests_rx).poll_next(cx) {
        Poll::Ready(Some((mut request, response_sender))) => {
          self.request_id += 1;
          // trace!("Got request from client: {:?}", request);
          request.id = self.request_id;
          trace!("=== Send Message to Service-serv: {:?}", request);
          stream.as_mut().start_send(Message::Request(request))?;
          self.pending_requests
            .insert(self.request_id, response_sender);
        }
        Poll::Ready(None) => {
          trace!("Client closed the requests channel.");
          self.client_closed = true;
          break;
        }
        Poll::Pending => {
          trace!("No new request from client");
          break;
        }
      }
    }
    Ok(())
  }

  fn process_response(&mut self, response: MsgPackResponse) {
    trace!("一个客户端的请求处理完成，response.id为{},\
            在pennding_requests中去掉这个id的key", &response.id);
    if let Some(response_tx) = self.pending_requests.remove(&response.id) {
      trace!("协程转发数据给客户端主线程 == Forwarding response to the client.");
      if let Err(e) = response_tx.send(response.result) {
        warn!("Failed to send response to client: {:?}", e);
      }
    } else {
      warn!("no pending request found for response {}", &response.id);
    }
  }

  fn acknowledge_notifications(&mut self) {
    for chan in self.pending_notifications.drain(..) {
      trace!("Acknowledging notification.");
      if let Err(e) = chan.send(()) {
        warn!("Failed to send ack to client: {:?}", e);
      }
    }
  }
}

struct Transport<T>(Framed<Compat<T>, Codec>);
// pub struct Transport<T>(Framed<Compat<T>, Codec>);

impl<T> Transport<T>
  where
    T: AsyncRead + AsyncWrite,
{
  fn inner(self: Pin<&mut Self>) -> Pin<&mut Framed<Compat<T>, Codec>> {
    unsafe { self.map_unchecked_mut(|this| &mut this.0) }
  }
}

impl<T> Stream for Transport<T>
  where
    T: AsyncRead + AsyncWrite,
{
  type Item = io::Result<Message>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
    trace!("Transport: polling");
    self.inner().poll_next(cx)
  }
}

impl<T> Sink<Message> for Transport<T>
  where
    T: AsyncRead + AsyncWrite,
{
  type Error = io::Error;

  fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    self.inner().poll_ready(cx)
  }

  fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
    self.inner().start_send(item)
  }

  fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    self.inner().poll_flush(cx)
  }

  fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
    self.inner().poll_close(cx)
  }
}

impl<S: Service> MessageHandler for Server<S> {
  // todo: server handle request data from client
  fn handle_incoming(&mut self, msg: Message) {
    trace!("=== impl MessageHandler for Server handle_incoming ===");
    match msg {
      Message::Request(req) => {
        // todo: invoke service method
        let f = self.service.handle_request(&req.method, &req.params);
        self.spawn_request_worker(req.id, f);
      }
      Message::Notification(note) => {
        self.service.handle_notification(&note.method, &note.params);
      }
      Message::Response(_) => {
        trace!("This endpoint doesn't handle responses, ignoring the msg.");
      }
    };
  }

  fn send_outgoing<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    sink: Pin<&mut Transport<T>>,
  ) -> Poll<io::Result<()>> {
    self.send_responses(cx, sink)
  }
}

impl MessageHandler for InnerClient {
  // todo: client handle response data from server
  fn handle_incoming(&mut self, msg: Message) {
    trace!("=== 接收到服务端数据, impl MessageHandler for InnerClient handle_incoming ===");
    trace!("handle_incoming Received {:?}", msg);
    if let Message::Response(response) = msg {
      self.process_response(response);
    } else {
      trace!("This endpoint only handles reponses, ignoring the msg.");
    }
  }

  fn send_outgoing<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    sink: Pin<&mut Transport<T>>,
  ) -> Poll<io::Result<()>> {
    trace!("=== impl MessageHandler for InnerClient invoke send_outgoing ===");
    self.send_messages(cx, sink)
  }

  fn is_finished(&self) -> bool {
    self.client_closed
      && self.pending_requests.is_empty()
      && self.pending_notifications.is_empty()
  }
}

struct ClientAndServer<S> {
  inner_client: InnerClient,
  server: Server<S>,
  client: Client,
}

impl<S: ServiceWithClient> MessageHandler for ClientAndServer<S> {
  fn handle_incoming(&mut self, msg: Message) {
    trace!("=== impl MessageHandler for ClientAndServer<S> handle_incoming ===");
    match msg {
      Message::Request(req) => {
        let f =
          self.server
            .service
            .handle_request(&mut self.client, &req.method, &req.params);
        self.server.spawn_request_worker(req.id, f);
      }
      Message::Notification(note) => {
        self.server.service.handle_notification(
          &mut self.client,
          &note.method,
          &note.params,
        );
      }
      Message::Response(response) => self.inner_client.process_response(response),
    };
  }

  fn send_outgoing<T: AsyncRead + AsyncWrite>(
    &mut self,
    cx: &mut Context,
    mut sink: Pin<&mut Transport<T>>,
  ) -> Poll<io::Result<()>> {
    if let Poll::Ready(()) = self.server.send_responses(cx, sink.as_mut())? {
      self.inner_client.send_messages(cx, sink)
    } else {
      Poll::Pending
    }
  }
}

struct InnerEndpoint<MH, T> {
  handler: MH,
  stream: Transport<T>,
}

// todo: ==step 2==
impl<MH: MessageHandler + Unpin, T: AsyncRead + AsyncWrite> Future for InnerEndpoint<MH, T> {
  type Output = io::Result<()>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    trace!("InnerEndpoint: polling");
    let (handler, mut stream) = unsafe {
      let this = self.get_unchecked_mut();
      (&mut this.handler, Pin::new_unchecked(&mut this.stream))
    };
    trace!("=== InnerEndpoint handler.send_outgoing, 客户端在这里发送数据给服务端! ===");
    if let Poll::Pending = handler.send_outgoing(cx, stream.as_mut())? {
      trace!("Sink not yet flushed, waiting...");
      return Poll::Pending;
    }

    trace!("=== 客户端Polling stream, 轮询stream, 也就是轮询socket事件, 接收服务端的返回! ===");
    // todo: stream.as_mut().poll_next go to ==> fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {}
    while let Poll::Ready(msg) = stream.as_mut().poll_next(cx)? {
      trace!("---check msg struct---");
      if let Some(msg) = msg {
        trace!("---handle_incoming msg---.");
        handler.handle_incoming(msg);
      } else {
        trace!("Stream closed by remote peer.");
        // FIXME: not sure if we should still continue sending responses here. Is it
        // possible that the client closed the stream only one way and is still waiting
        // for response? Not for TCP at least, but maybe for other transport types?
        return Poll::Ready(Ok(()));
      }
    }

    if handler.is_finished() {
      trace!("inner client finished, exiting...");
      Poll::Ready(Ok(()))
    } else {
      trace!("notifying the reactor that we're not done yet");
      trace!("=== 这里执行 Poll:Pending, 如果客户端已经没有发送数据给服务端的话，那就是不会触发 InnerEndpoint: polling(通信入口的轮询), 客户端就不会polling socket事件， 客户端程序会退出 ===");
      Poll::Pending
    }
  }
}

/// Creates a future for running a `Service` on a stream.
///
/// The returned future will run until the stream is closed; if the stream encounters an error,
/// then the future will propagate it and terminate.
pub fn serve<'a, S: Service + Unpin + 'a, T: AsyncRead + AsyncWrite + 'a + Send>(
  stream: T,
  service: S,
) -> impl Future<Output = io::Result<()>> + 'a + Send {
  ServerEndpoint::new(stream, service)
}

struct ServerEndpoint<S, T> {
  inner: InnerEndpoint<Server<S>, T>,
}

impl<S: Service + Unpin, T: AsyncRead + AsyncWrite> ServerEndpoint<S, T> {
  pub fn new(stream: T, service: S) -> Self {
    let stream = FuturesAsyncWriteCompatExt::compat_write(stream);
    ServerEndpoint {
      inner: InnerEndpoint {
        stream: Transport(Codec.framed(stream)),
        handler: Server::new(service),
      },
    }
  }
}

impl<S: Service + Unpin, T: AsyncRead + AsyncWrite> Future for ServerEndpoint<S, T> {
  type Output = io::Result<()>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    trace!("ServerEndpoint: polling");
    unsafe { self.map_unchecked_mut(|this| &mut this.inner) }.poll(cx)
  }
}

pub struct Endpoint<S, T> {
  inner: InnerEndpoint<ClientAndServer<S>, T>,
}

impl<S: ServiceWithClient + Unpin, T: AsyncRead + AsyncWrite> Endpoint<S, T> {
  /// Creates a new `Endpoint` on `stream`, using `service` to handle requests and notifications.
  pub fn new(stream: T, service: S) -> Self {
    let (inner_client, client) = InnerClient::new();
    let stream = FuturesAsyncWriteCompatExt::compat_write(stream);
    Endpoint {
      inner: InnerEndpoint {
        stream: Transport(Codec.framed(stream)),
        handler: ClientAndServer {
          inner_client,
          client,
          server: Server::new(service),
        },
      },
    }
  }

  /// Returns a handle to the client half of this `Endpoint`, which can be used for sending
  /// requests and notifications.
  pub fn client(&self) -> Client {
    self.inner.handler.client.clone()
  }
}

impl<S: ServiceWithClient + Unpin, T: AsyncRead + AsyncWrite> Future for Endpoint<S, T> {
  type Output = io::Result<()>;

  fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    trace!("Endpoint: polling");
    unsafe { self.map_unchecked_mut(|this| &mut this.inner) }.poll(cx)
  }
}

/// A client that sends requests and notifications to a remote MessagePack-RPC server.
#[derive(Clone)]
pub struct Client {
  requests_tx: RequestTx,
  notifications_tx: NotificationTx,
}

impl Client {

  // todo: T inherit reait for AsyncRead, AsyncWrite etc.
  // todo: the steps for client call server
  // todo: 1. client.call fn call(&self, method: &str, params: &[Value]) -> Response {}
  // todo: 2. impl Future for InnerEndpoint<MH, T>, invoke the poll fn
  // todo: 3. because 1 return Response, so step 3 is impl Future for Response{}, invoke poll fn
  // todo: 4. impl Future for InnerEndpoint, invoke poll fn, in this code process `let Poll::Pending = handler.send_outgoing(cx, stream.as_mut())`
  pub fn new<T: AsyncRead + AsyncWrite + 'static + Send>(stream: T) -> Self {
    let (inner_client, client) = InnerClient::new();
    let stream = FuturesAsyncWriteCompatExt::compat_write(stream);
    let endpoint = InnerEndpoint {
      stream: Transport(Codec.framed(stream)),
      handler: inner_client,
    };
    // We swallow io::Errors. The client will see an error if it has any outstanding requests
    // or if it tries to send anything, because the endpoint has terminated.
    // todo: 这里不是异步检查 endpoint是否有错误? 而且异步跑
    tokio::task::spawn(
      endpoint.map_err(|e| error!("Client endpoint closed because of an error: {}", e)),
    );

    client
  }

  // fn from_channels(requests_tx: RequestTx, notifications_tx: NotificationTx) -> Self {
  //   Client {
  //     requests_tx,
  //     notifications_tx,
  //   }
  // }

  /// Send a `MessagePack-RPC` request
  // todo: ==step 1==
  pub fn call(&self, method: &str, params: &[Value]) -> Response {
    trace!("New call (method={}, params={:?})", method, params);
    let request = Request {
      id: 0,
      method: method.to_owned(),
      params: Vec::from(params),
    };
    let (tx, rx) = oneshot::channel();
    // If send returns an Err, its because the other side has been dropped. By ignoring it,
    // we are just dropping the `tx`, which will mean the rx will return Canceled when
    // polled. In turn, that is translated into a BrokenPipe, which conveys the proper
    // error.
    // todo: this will trigger the InnerEndpoint poll fn
    let _ = mpsc::UnboundedSender::unbounded_send(&self.requests_tx, (request, tx));
    Response(rx)
  }

  /// Send a `MessagePack-RPC` notification
  pub fn call_notify(&self, method: &str, params: &[Value]) -> Ack {
    trace!("New notification (method={}, params={:?})", method, params);
    let notification = Notification {
      method: method.to_owned(),
      params: Vec::from(params),
    };
    let (tx, rx) = oneshot::channel();
    let _ = mpsc::UnboundedSender::unbounded_send(&self.notifications_tx, (notification, tx));
    Ack(rx)
  }
}

impl Future for Client {
  type Output = io::Result<()>;

  fn poll(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
    trace!("Client: polling");
    Poll::Ready(Ok(()))
  }
}


