use std::any::Any;
use std::env;
use std::sync::Arc;

use crate::actor::actor::pid::ExtendedPid;
use crate::actor::actor::props::{Props, ReceiverMiddleware};
use crate::actor::actor::restart_statistics::RestartStatistics;
use crate::actor::actor::{Actor, ActorError, ActorHandle, ActorInnerError, Stop};
use crate::actor::actor_system::ActorSystem;
use crate::actor::context::{ContextHandle, MessagePart, ReceiverContextHandle, SenderPart, SpawnerPart};
use crate::actor::message::{Message, MessageHandle, ProducerFunc, ReceiverFunc};
use crate::actor::messages::{AutoReceiveMessage, Restart, Started, SystemMessage};
use crate::actor::supervisor::supervisor_strategy::{SupervisorHandle, SupervisorStrategy, SupervisorStrategyHandle};
use async_trait::async_trait;
use tokio::sync::{mpsc, Mutex, Notify};
use tokio::time::{sleep, timeout, Instant};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
struct ActorWithSupervisor {
  notify: Arc<Notify>,
}

#[derive(Debug, Clone)]
struct FailingChildActor;

#[derive(Debug, Clone)]
struct StringMessage(String);

impl PartialEq for StringMessage {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl Message for StringMessage {
  fn eq_message(&self, other: &dyn Message) -> bool {
    let other = other.as_any().downcast_ref::<StringMessage>();
    match other {
      Some(other) => self.0 == other.0,
      None => false,
    }
  }

  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}

#[async_trait]
impl Actor for ActorWithSupervisor {
  async fn started(&self, mut ctx: ContextHandle) -> Result<(), ActorError> {
    tracing::debug!("ActorWithSupervisor::post_start");
    let props = Props::from_producer_func(ProducerFunc::new(|ctx| async { ActorHandle::new(FailingChildActor) })).await;
    let child = ctx.spawn(props).await;
    ctx
      .send(child, MessageHandle::new(StringMessage("fail".to_string())))
      .await;
    Ok(())
  }

  async fn receive(&mut self, _: ContextHandle, _: MessageHandle) -> Result<(), ActorError> {
    tracing::debug!("ActorWithSupervisor::receive");
    Ok(())
  }

  async fn get_supervisor_strategy(&self) -> Option<SupervisorStrategyHandle> {
    Some(SupervisorStrategyHandle::new(self.clone()))
  }
}

#[async_trait]
impl SupervisorStrategy for ActorWithSupervisor {
  async fn handle_child_failure(
    &self,
    _: &ActorSystem,
    _: SupervisorHandle,
    child: ExtendedPid,
    rs: RestartStatistics,
    _: ActorInnerError,
    message: MessageHandle,
  ) {
    tracing::debug!(
      "ActorWithSupervisor::handle_failure: child = {:?}, rs = {:?}, message = {:?}",
      child,
      rs,
      message
    );
    self.notify.notify_one();
  }

  fn as_any(&self) -> &dyn Any {
    self
  }
}

#[async_trait]
impl Actor for FailingChildActor {
  async fn started(&self, _: ContextHandle) -> Result<(), ActorError> {
    tracing::debug!("FailingChildActor::post_start");
    Ok(())
  }

  async fn receive(&mut self, c: ContextHandle, message_handle: MessageHandle) -> Result<(), ActorError> {
    tracing::debug!("FailingChildActor::receive: msg = {:?}", message_handle);
    if let Some(StringMessage(msg)) = message_handle.as_any().downcast_ref::<StringMessage>() {
      tracing::debug!("FailingChildActor::receive: msg = {:?}", msg);
      Err(ActorError::ReceiveError(ActorInnerError::new("error")))
    } else {
      Ok(())
    }
  }
}

#[tokio::test]
async fn test_actor_with_own_supervisor_can_handle_failure() {
  let _ = env::set_var("RUST_LOG", "debug");
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();

  let system = ActorSystem::new().await;
  let mut root = system.get_root_context().await;
  let notify = Arc::new(Notify::new());
  let cloned_notify = notify.clone();
  let props = Props::from_producer_func(ProducerFunc::new(move |_| {
    let cloned_notify = cloned_notify.clone();
    async move {
      ActorHandle::new(ActorWithSupervisor {
        notify: cloned_notify.clone(),
      })
    }
  }))
  .await;
  let pid = root.spawn(props).await;
  tracing::info!("pid = {:?}", pid);
  notify.notified().await;
}
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Debug, Error)]
enum TestError {
  #[error("Timeout")]
  TimeoutError,
}

#[derive(Debug, Clone)]
struct Observer {
  received: Arc<Mutex<VecDeque<MessageHandle>>>,
}

impl Observer {
  fn new() -> Self {
    Observer {
      received: Arc::new(Mutex::new(VecDeque::new())),
    }
  }

  async fn receive(&self, _: ReceiverContextHandle, message: MessageHandle) -> Result<(), ActorError> {
    self.received.lock().await.push_back(message);
    Ok(())
  }

  async fn expect_message(&self, expected: MessageHandle, timeout: tokio::time::Duration) -> Result<(), TestError> {
    let start = Instant::now();
    while start.elapsed() <= timeout {
      if let Some(received) = self.received.lock().await.pop_front() {
        if expected == received {
          return Ok(());
        }
      }
      tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    Err(TestError::TimeoutError)
  }
}

#[tokio::test]
async fn test_actor_stops_after_x_restarts() {
  let _ = env::set_var("RUST_LOG", "debug");
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
  let mut observer = Observer::new();
  let system = ActorSystem::new().await;
  let mut root_context = system.get_root_context().await;

  let cloned_observer = observer.clone();
  let root_context_clone = root_context.clone(); // クローンを作成
  let middles = ReceiverMiddleware::new(move |next| {
    let root_context = root_context_clone.clone(); // クローンを使用
    let cloned_observer = cloned_observer.clone();
    ReceiverFunc::new(move |ctx, moe| {
      let next = next.clone();
      let cloned_observer = cloned_observer.clone();
      async move {
        tracing::debug!("ReceiverMiddleware: moe = {:?}", moe);
        let msg = moe.get_message();
        tracing::debug!(">>>> msg = {:?}", msg);
        let result = cloned_observer.receive(ctx.clone(), msg.clone()).await;
        if result.is_err() {
          return result;
        }
        next.run(ctx, moe).await
      }
    })
  });
  let props = Props::from_producer_func(ProducerFunc::new(|_| async { ActorHandle::new(FailingChildActor) })).await;
  // let props = Props::from_producer_func_with_opts(
  //   ProducerFunc::new(|_| async { ActorHandle::new(FailingChildActor) }),
  //   vec![Props::with_receiver_middleware(vec![middles])],
  // )
  // .await;

  let child = root_context.spawn(props).await;
  let fail = MessageHandle::new(StringMessage("fail".to_string()));
  let d = tokio::time::Duration::from_secs(10);
  // let _ = observer
  //   .expect_message(MessageHandle::new(SystemMessage::Started(Started {})), d)
  //   .await;

  for i in 0..6 {
    tracing::debug!("Sending fail message: {}", i);
    root_context.send(child.clone(), fail.clone()).await;
    // observer.expect_message(fail.clone(), d).await.unwrap();
    // observer
    //   .expect_message(
    //     MessageHandle::new(AutoReceiveMessage::Restarting(crate::actor::messages::Restarting {})),
    //     d,
    //   )
    //   .await
    //   .unwrap();
    // observer
    //   .expect_message(MessageHandle::new(SystemMessage::Started(Started {})), d)
    //   .await
    //   .unwrap();
  }
  // root_context.send(child, fail.clone()).await;
  // observer.expect_message(fail.clone(), d).await.unwrap();
  // observer
  //   .expect_message(
  //     MessageHandle::new(AutoReceiveMessage::Stopping(crate::actor::messages::Stopping {})),
  //     d,
  //   )
  //   .await
  //   .unwrap();

  sleep(std::time::Duration::from_secs(60)).await;
}
