#[cfg(test)]
mod test {
  use std::env;
  use std::time::Duration;

  use async_trait::async_trait;
  use tokio::sync::mpsc;
  use tokio::time::sleep;
  use tracing_subscriber::EnvFilter;

  use crate::actor::actor::actor::Actor;
  use crate::actor::actor::actor_error::ActorError;

  use crate::actor::actor::actor_inner_error::ActorInnerError;
  use crate::actor::actor::props::Props;
  use crate::actor::actor_system::ActorSystem;
  use crate::actor::context::context_handle::ContextHandle;
  use crate::actor::context::{SenderPart, SpawnerPart};
  use crate::actor::message::message::Message;
  use crate::actor::message::message_handle::MessageHandle;
  use crate::actor::supervisor::exponential_backoff_strategy::ExponentialBackoffStrategy;
  use crate::actor::supervisor::strategy_all_for_one::AllForOneStrategy;
  use crate::actor::supervisor::strategy_one_for_one::OneForOneStrategy;
  use crate::actor::supervisor::strategy_restarting::RestartingStrategy;
  use crate::actor::supervisor::supervision_event::SupervisorEvent;
  use crate::actor::supervisor::supervisor_strategy_handle::SupervisorStrategyHandle;
  use crate::event_stream::event_handler::EventHandler;

  #[derive(Debug)]
  struct PanicActor;

  #[async_trait]
  impl Actor for PanicActor {
    async fn receive(&mut self, _: ContextHandle, message_handle: MessageHandle) -> Result<(), ActorError> {
      if message_handle.to_typed::<String>().is_some() {
        Err(ActorError::ReceiveError(ActorInnerError::new("Boom!".to_string())))
      } else {
        Ok(())
      }
    }
  }

  #[tokio::test]
  async fn test_supervisor_event_handle_from_event_stream() {
    let _ = env::set_var("RUST_LOG", "debug");
    let _ = tracing_subscriber::fmt()
      .with_env_filter(EnvFilter::from_default_env())
      .try_init();

    let supervisors = vec![
      (
        "all_for_one",
        SupervisorStrategyHandle::new(AllForOneStrategy::new(10, Duration::from_secs(10))),
      ),
      (
        "exponential_backoff",
        SupervisorStrategyHandle::new(ExponentialBackoffStrategy::new(Duration::from_millis(10))),
      ),
      (
        "one_for_one",
        SupervisorStrategyHandle::new(OneForOneStrategy::new(10, Duration::from_secs(10))),
      ),
      ("restarting", SupervisorStrategyHandle::new(RestartingStrategy::new())),
    ];

    for (_, strategy) in supervisors {
      let system = ActorSystem::new().await;
      let (tx, mut rx) = mpsc::channel(100);

      system
        .get_event_stream()
        .await
        .subscribe(EventHandler::new(move |evt| {
          let tx = tx.clone();
          async move {
            if evt.as_any().downcast_ref::<SupervisorEvent>().is_some() {
              tx.try_send(()).unwrap();
            }
          }
        }))
        .await;

      let props = Props::from_actor_producer_with_opts(
        move |_| async { PanicActor },
        [Props::with_supervisor_strategy(strategy.clone())],
      )
      .await;

      let mut root_context = system.get_root_context().await;
      let pid = root_context.spawn(props).await;

      root_context.send(pid, MessageHandle::new("Fail!".to_string())).await;

      tokio::select! {
          _ = rx.recv() => {},
          _ = sleep(Duration::from_secs(5)) => {
              panic!("Timeout waiting for SupervisorEvent");
          }
      }
    }
  }
}
