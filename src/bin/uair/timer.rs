use std::io::{self, Stdout, Write};
use std::fmt::Write as _;
use std::time::{Duration, Instant};
use async_io::Timer;
use crate::Error;
use crate::app::Event;
use crate::session::Session;
use crate::socket::BlockingStream;

pub struct UairTimer {
	interval: Duration,
	streams: Vec<(BlockingStream, Option<String>)>,
	stdout: Stdout,
	buf: String,
	pub state: State,
}

impl UairTimer {
	pub fn new(interval: Duration) -> Self {
		UairTimer {
			stdout: io::stdout(),
			interval,
			streams: Vec::new(),
			buf: "".into(),
			state: State::Paused(Duration::ZERO),
		}
	}

	pub async fn start(&mut self, session: &Session, start: Instant, dest: Instant) -> Result<Event, Error> {
		let duration = dest - start;
		let first_interval = Duration::from_nanos(duration.subsec_nanos().into());
		let mut end = start + first_interval;

		while end <= dest {
			Timer::at(end).await;
			self.write::<true>(session, dest - end)?;
			end += self.interval;
		}

		Ok(Event::Finished)
	}

	pub fn write<const R: bool>(&mut self, session: &Session, duration: Duration) -> Result<(), Error> {
		write!(self.buf, "{}", session.display::<R>(duration, None))?;
		write!(self.stdout, "{}", self.buf)?;
		self.stdout.flush()?;
		self.buf.clear();
		self.streams.retain_mut(|(stream, overrid)| {
			let overrid = overrid.as_ref().and_then(|o| session.overrides.get(o));
			if write!(self.buf, "{}\0", session.display::<R>(duration, overrid)).is_err() {
				self.buf += "Formatting Error";
			}
			let res = stream.write(self.buf.as_bytes()).is_ok();
			self.buf.clear();
			res
		});
		Ok(())
	}

	pub fn add_stream(&mut self, stream: BlockingStream, overrid: Option<String>) {
		self.streams.push((stream, overrid));
	}
}

impl Drop for UairTimer {
	fn drop(&mut self) {
		if let State::Resumed(_, dest) = self.state {
			self.state = State::Resumed(Instant::now(), dest)
		}
	}
}

pub enum State {
	Paused(Duration),
	Resumed(Instant, Instant),
	Finished,
}
