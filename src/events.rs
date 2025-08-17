use anyhow::Result;
use crossterm::event::{self, Event};
use std::time::Duration;
use tokio::time::timeout;

pub struct EventHandler {
    #[allow(dead_code)]
    tick_rate: Duration,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        // Poll for events with a timeout to prevent blocking indefinitely
        let event = timeout(Duration::from_millis(100), async {
            loop {
                if event::poll(Duration::from_millis(0))? {
                    return Ok(event::read()?);
                }
                tokio::task::yield_now().await;
            }
        })
        .await;

        match event {
            Ok(event_result) => event_result,
            Err(_) => {
                // Timeout occurred, return a dummy event
                // This allows the main loop to continue and handle other async tasks
                Ok(Event::Resize(0, 0)) // This will be filtered out in the main loop
            }
        }
    }
}
