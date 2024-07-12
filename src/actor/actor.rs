pub mod actor;
pub mod actor_error;
pub mod actor_handle;
pub mod actor_inner_error;
pub mod actor_process;
pub mod actor_producer;
pub mod actor_receiver;
pub mod behavior;
pub mod context_decorator;
pub mod context_decorator_chain;
pub mod context_handler;
pub mod continuer;
pub mod middleware_chain;
pub mod pid;
pub mod pid_set;
mod pid_set_test;
pub mod props;
pub mod receiver_middleware;
pub mod receiver_middleware_chain;
pub mod restart_statistics;
pub mod sender_middleware;
pub mod sender_middleware_chain;
pub mod spawn_middleware;
pub mod spawner;
pub mod taks;

// include!(concat!(env!("OUT_DIR"), "/actor.rs"));
// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pid {
  #[prost(string, tag = "1")]
  pub address: ::prost::alloc::string::String,
  #[prost(string, tag = "2")]
  pub id: ::prost::alloc::string::String,
  #[prost(uint32, tag = "3")]
  pub request_id: u32,
}
/// user messages

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeadLetterResponse {
  #[prost(message, optional, tag = "1")]
  pub target: ::core::option::Option<Pid>,
}
/// system messages

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Terminated {
  #[prost(message, optional, tag = "1")]
  pub who: ::core::option::Option<Pid>,
  #[prost(enumeration = "TerminatedReason", tag = "2")]
  pub why: i32,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct Touch {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Touched {
  #[prost(message, optional, tag = "1")]
  pub who: ::core::option::Option<Pid>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TerminatedReason {
  Stopped = 0,
  AddressTerminated = 1,
  NotFound = 2,
}
impl TerminatedReason {
  /// String value of the enum field names used in the ProtoBuf definition.
  ///
  /// The values are not transformed in any way and thus are considered stable
  /// (if the ProtoBuf definition does not change) and safe for programmatic use.
  pub fn as_str_name(&self) -> &'static str {
    match self {
      TerminatedReason::Stopped => "Stopped",
      TerminatedReason::AddressTerminated => "AddressTerminated",
      TerminatedReason::NotFound => "NotFound",
    }
  }

  /// Creates an enum from field names used in the ProtoBuf definition.
  pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
    match value {
      "Stopped" => Some(Self::Stopped),
      "AddressTerminated" => Some(Self::AddressTerminated),
      "NotFound" => Some(Self::NotFound),
      _ => None,
    }
  }
}
