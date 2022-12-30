use std::time::Duration;
use async_io::Timer;
use crate::Error;
use crate::app::Event;
use crate::session::Session;

pub struct UairTimer {
	pub duration: Duration,
	interval: Duration,
}

impl UairTimer {
	pub fn new(duration: Duration, interval: Duration) -> Self {
		UairTimer { duration, interval }
	}

	pub async fn start(&mut self, session: &Session) -> Result<Event, Error> {
		while self.duration > Duration::ZERO {
			Timer::after(self.interval).await;
			self.duration -= self.interval;
			session.display::<true>(self.duration)?;
		}

		Ok(Event::Finished)
	}
}
