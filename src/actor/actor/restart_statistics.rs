use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RestartStatistics {
  failure_times: Arc<Mutex<Vec<Instant>>>,
}

impl RestartStatistics {
  pub fn new() -> Self {
    RestartStatistics {
      failure_times: Arc::new(Mutex::new(vec![])),
    }
  }

  pub async fn failure_count(&self) -> usize {
    let mg = self.failure_times.lock().await;
    mg.len()
  }

  pub async fn fail(&mut self) {
    let mut mg = self.failure_times.lock().await;
    mg.push(Instant::now());
  }

  pub async fn reset(&mut self) {
    let mut mg = self.failure_times.lock().await;
    mg.clear();
  }

  pub async fn number_of_failures(&self, within_duration: Duration) -> u32 {
    if within_duration == Duration::ZERO {
      let mg = self.failure_times.lock().await;
      return mg.len() as u32;
    }

    let curr_time = Instant::now();
    let mg = self.failure_times.lock().await;
    mg
      .iter()
      .filter(|&&t| curr_time.duration_since(t) < within_duration)
      .count() as u32
  }
}

impl Default for RestartStatistics {
  fn default() -> Self {
    Self::new()
  }
}
