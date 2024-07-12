#[cfg(test)]
mod tests {
  use std::env;

  use tracing_subscriber::EnvFilter;

  use crate::actor::actor::actor_receiver::ActorReceiver;
  use crate::actor::actor::props::Props;
  use crate::actor::actor_system::ActorSystem;
  use crate::actor::context::{MessagePart, SpawnerPart};
  use crate::actor::message::message::Message;
  use crate::actor::message::system_message::SystemMessage;
  use crate::actor::util::async_barrier::AsyncBarrier;

  #[tokio::test]
  async fn example_root_context_spawn() {
    let _ = env::set_var("RUST_LOG", "debug");
    let _ = tracing_subscriber::fmt()
      .with_env_filter(EnvFilter::from_default_env())
      .try_init();
    let b = AsyncBarrier::new(2);

    let system = ActorSystem::new().await;
    let cloned_b = b.clone();

    let props = Props::from_actor_receiver(ActorReceiver::new(move |ctx| {
      let b = cloned_b.clone();
      async move {
        let msg = ctx.get_message_handle().await;
        tracing::debug!("msg = {:?}", msg);
        if let Some(SystemMessage::Started) = msg.as_any().downcast_ref::<SystemMessage>() {
          tracing::debug!("Hello World!");
          b.wait().await;
        }
        Ok(())
      }
    }))
    .await;

    let pid = system.get_root_context().await.spawn(props).await;
    tracing::debug!("pid = {:?}", pid);

    b.wait().await;
  }
}
