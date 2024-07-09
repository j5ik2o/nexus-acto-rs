use std::any::Any;
use std::fmt::Debug;

use crate::actor::actor::pid::ExtendedPid;
use crate::actor::message::message::Message;
use crate::actor::message::message_handle::MessageHandle;
use crate::actor::message::message_headers::MessageHeaders;
use crate::actor::message::readonly_message_headers::ReadonlyMessageHeaders;
use crate::actor::message::system_message::SystemMessage;

#[derive(Debug, Clone)]
pub struct MessageEnvelope {
  header: Option<MessageHeaders>,
  message_handle: MessageHandle,
  sender: Option<ExtendedPid>,
}

impl PartialEq for MessageEnvelope {
  fn eq(&self, other: &Self) -> bool {
    self.header == other.header && self.message_handle == other.message_handle && self.sender == other.sender
  }
}

impl Message for MessageEnvelope {
  fn eq_message(&self, other: &dyn Message) -> bool {
    if let Some(other) = other.as_any().downcast_ref::<Self>() {
      self.header == other.header && self.message_handle == other.message_handle && self.sender == other.sender
    } else {
      false
    }
  }

  fn as_any(&self) -> &(dyn Any + Send + Sync + 'static) {
    self
  }
}

impl MessageEnvelope {
  pub fn new(message_handle: MessageHandle) -> Self {
    if message_handle.as_any().is::<SystemMessage>() {
      tracing::warn!("SystemMessage can't be used as a message, {:?}", message_handle);
    }
    Self {
      header: None,
      message_handle: message_handle,
      sender: None,
    }
  }

  pub fn with_header(mut self, header: MessageHeaders) -> Self {
    self.header = Some(header);
    self
  }

  pub fn with_sender(mut self, sender: ExtendedPid) -> Self {
    self.sender = Some(sender);
    self
  }

  pub fn get_message_handle(&self) -> MessageHandle {
    self.message_handle.clone()
  }

  pub fn get_sender(&self) -> Option<ExtendedPid> {
    self.sender.clone()
  }

  pub fn get_header(&self) -> Option<MessageHeaders> {
    self.header.clone()
  }

  pub fn get_header_value(&self, key: &str) -> Option<String> {
    self.header.as_ref().and_then(|h| h.get(key).cloned())
  }

  pub fn set_header(&mut self, key: String, value: String) {
    if self.header.is_none() {
      self.header = Some(MessageHeaders::default());
    }
    if let Some(h) = &mut self.header {
      h.set(key, value);
    }
  }

  pub fn get_headers(&self) -> Option<MessageHeaders> {
    self.header.clone()
  }
}

pub fn wrap_envelope(message_handle: MessageHandle) -> MessageEnvelope {
  if let Some(envelope) = message_handle.to_typed::<MessageEnvelope>() {
    envelope.clone()
  } else {
    MessageEnvelope::new(message_handle)
  }
}

pub fn unwrap_envelope(message_handle: MessageHandle) -> (Option<MessageHeaders>, MessageHandle, Option<ExtendedPid>) {
  if let Some(envelope) = message_handle.to_typed::<MessageEnvelope>() {
    (
      envelope.header.clone(),
      envelope.message_handle.clone(),
      envelope.sender.clone(),
    )
  } else {
    (None, message_handle, None)
  }
}

pub fn unwrap_envelope_header(message_handle: MessageHandle) -> Option<MessageHeaders> {
  if let Some(envelope) = message_handle.to_typed::<MessageEnvelope>() {
    envelope.header.clone().map(|h| MessageHeaders::with_values(h.to_map()))
  } else {
    None
  }
}

pub fn unwrap_envelope_message(message_handle: MessageHandle) -> MessageHandle {
  if let Some(envelope) = message_handle.to_typed::<MessageEnvelope>() {
    envelope.message_handle.clone()
  } else {
    message_handle
  }
}

pub fn unwrap_envelope_sender(message_handle: MessageHandle) -> Option<ExtendedPid> {
  if let Some(envelope) = message_handle.to_typed::<MessageEnvelope>() {
    envelope.sender.clone()
  } else {
    // tracing::debug!("unwrap_envelope_sender: None");
    None
  }
}
