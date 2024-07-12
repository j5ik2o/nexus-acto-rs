use std::any::Any;
use std::fmt::Display;

use crate::actor::actor::pid::ExtendedPid;
use crate::actor::message::message::Message;
use crate::actor::message::message_handle::MessageHandle;

#[derive(Debug, Clone)]
pub enum AutoReceiveMessage {
  PostStart,
  PreRestart,
  PreStop,
  PostStop,
  PoisonPill,
}

impl AutoReceiveMessage {
  pub fn auto_receive_message(&self, _: &ExtendedPid, _: MessageHandle) {}
}

static_assertions::assert_impl_all!(AutoReceiveMessage: Send, Sync);

impl PartialEq for AutoReceiveMessage {
  fn eq(&self, other: &Self) -> bool {
    self.eq_message(other)
  }
}

impl Eq for AutoReceiveMessage {}

impl Display for AutoReceiveMessage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AutoReceiveMessage::PostStart => write!(f, "PostStart"),
      AutoReceiveMessage::PreRestart => write!(f, "PreRestart"),
      AutoReceiveMessage::PreStop => write!(f, "PreStop"),
      AutoReceiveMessage::PostStop => write!(f, "PostStop"),
      AutoReceiveMessage::PoisonPill => write!(f, "PoisonPill"),
    }
  }
}

impl Message for AutoReceiveMessage {
  fn eq_message(&self, other: &dyn Message) -> bool {
    let msg = other.as_any().downcast_ref::<AutoReceiveMessage>();
    match (self, msg) {
      (AutoReceiveMessage::PostStart, Some(&AutoReceiveMessage::PostStart)) => true,
      (AutoReceiveMessage::PreRestart, Some(&AutoReceiveMessage::PreRestart)) => true,
      (AutoReceiveMessage::PreStop, Some(&AutoReceiveMessage::PreStop)) => true,
      (AutoReceiveMessage::PostStop, Some(&AutoReceiveMessage::PostStop)) => true,
      (AutoReceiveMessage::PoisonPill, Some(&AutoReceiveMessage::PoisonPill)) => true,
      _ => false,
    }
  }

  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}
