#[cfg(test)]
mod tests {
  use std::sync::{Arc, RwLock};

  use crate::log::log::{Level, Logger};
  use crate::log::log_event::LogEvent;
  use crate::log::log_event_stream::{publish_to_stream, subscribe_stream, unsubscribe_stream, LogEventStream};
  use crate::log::log_field::LogField;

  #[tokio::test]
  async fn test_logger_with() {
    let event_stream = LogEventStream::new();
    let base = Logger::new(event_stream, Level::Debug, "").with_fields([LogField::string("first", "value")]);
    let l = base.with_fields([LogField::string("second", "value")]);

    assert_eq!(
      vec![LogField::string("first", "value"), LogField::string("second", "value")],
      l.get_context()
    );
  }

  #[tokio::test]
  async fn test_off_level_two_fields() {
    let event_stream = LogEventStream::new();
    let l = Logger::new(event_stream, Level::Min, "");
    l.debug_with_fields("foo", [LogField::int("bar", 32), LogField::bool("fum", false)])
      .await;
  }

  #[tokio::test]
  async fn test_off_level_only_context() {
    let event_stream = LogEventStream::new();
    let l =
      Logger::new(event_stream, Level::Min, "").with_fields([LogField::int("bar", 32), LogField::bool("fum", false)]);
    l.debug("foo").await;
  }

  #[tokio::test]
  async fn test_debug_level_only_context_one_subscriber() {
    let event_stream = LogEventStream::new();
    let _s1 = subscribe_stream(&event_stream, |_: LogEvent| async {}).await;

    let l =
      Logger::new(event_stream, Level::Debug, "").with_fields([LogField::int("bar", 32), LogField::bool("fum", false)]);
    l.debug("foo").await;

    unsubscribe_stream(&_s1).await;
  }

  #[tokio::test]
  async fn test_debug_level_only_context_multiple_subscribers() {
    let event_stream = LogEventStream::new();
    let _s1 = subscribe_stream(&event_stream, |_: LogEvent| async {}).await;
    let _s2 = subscribe_stream(&event_stream, |_: LogEvent| async {}).await;

    let l =
      Logger::new(event_stream, Level::Debug, "").with_fields([LogField::int("bar", 32), LogField::bool("fum", false)]);
    l.debug("foo").await;

    unsubscribe_stream(&_s1).await;
    unsubscribe_stream(&_s2).await;
  }

  #[tokio::test]
  async fn test_subscribe_and_publish() {
    let event_stream = LogEventStream::new();
    let received = Arc::new(RwLock::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let sub = subscribe_stream(&event_stream, move |evt: LogEvent| {
      let received_clone = received_clone.clone();
      async move {
        received_clone.write().unwrap().push(evt.message.clone());
      }
    })
    .await;

    publish_to_stream(&event_stream, LogEvent::new(Level::Info, "Test message".to_string())).await;

    assert_eq!(received.read().unwrap().len(), 1);
    assert_eq!(received.read().unwrap()[0], "Test message");

    unsubscribe_stream(&sub).await;
  }

  #[tokio::test]
  async fn test_min_level_filtering() {
    let event_stream = LogEventStream::new();
    let received = Arc::new(RwLock::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let sub = subscribe_stream(&event_stream, move |evt: LogEvent| {
      let received_clone = received_clone.clone();
      async move {
        received_clone.write().unwrap().push(evt.message.clone());
      }
    })
    .await
    .with_min_level(Level::Warn);

    publish_to_stream(&event_stream, LogEvent::new(Level::Info, "Info message".to_string())).await;
    publish_to_stream(&event_stream, LogEvent::new(Level::Warn, "Warn message".to_string())).await;
    publish_to_stream(&event_stream, LogEvent::new(Level::Error, "Error message".to_string())).await;

    assert_eq!(received.read().unwrap().len(), 2);
    assert_eq!(received.read().unwrap()[0], "Warn message");
    assert_eq!(received.read().unwrap()[1], "Error message");

    unsubscribe_stream(&sub).await;
  }
}
