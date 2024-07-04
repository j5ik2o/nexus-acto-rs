use std::fmt::Debug;
use std::sync::Arc;

use crate::actor::context::context_handle::ContextHandle;
use crate::actor::message::response::ResponseHandle;

pub trait AutoRespond: Debug + Send + Sync + 'static {
  fn get_auto_response(&self, context: ContextHandle) -> ResponseHandle;
}

#[derive(Debug, Clone)]
pub struct AutoRespondHandle(Arc<dyn AutoRespond>);

impl AutoRespondHandle {
  pub fn new(auto_respond: Arc<dyn AutoRespond>) -> Self {
    Self(auto_respond)
  }
}

impl AutoRespond for AutoRespondHandle {
  fn get_auto_response(&self, context: ContextHandle) -> ResponseHandle {
    self.0.get_auto_response(context)
  }
}
