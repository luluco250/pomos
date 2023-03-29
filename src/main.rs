use cancellable_timer::Timer;
use std::{
	io,
	process::exit,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
enum PomodoroStep {
	Work,
	ShortBreak,
	LongBreak,
}

struct PomodoroTimer<Sleep>
where
	Sleep: FnMut(Duration) -> (),
{
	work_duration: Duration,
	short_break_duration: Duration,
	long_break_duration: Duration,
	_sleep: Sleep,
	_step: PomodoroStep,
	_counter: u8,
}

impl<Sleep> PomodoroTimer<Sleep>
where
	Sleep: FnMut(Duration) -> (),
{
	pub fn new(sleep: Sleep) -> PomodoroTimer<Sleep> {
		PomodoroTimer::<Sleep> {
			work_duration: Duration::from_secs(60 * 25),
			short_break_duration: Duration::from_secs(60 * 5),
			long_break_duration: Duration::from_secs(60 * 30),
			_sleep: sleep,
			_step: PomodoroStep::ShortBreak,
			_counter: 0,
		}
	}

	pub fn next(&mut self) {
		self._counter += 1;

		if self._counter % 2 == 1 {
			self._step = PomodoroStep::Work;
			return;
		}

		if self._counter % 6 == 0 {
			self._counter = 0;
			self._step = PomodoroStep::LongBreak;
			return;
		}

		self._step = PomodoroStep::ShortBreak;
	}

	pub fn sleep(&mut self) {
		(self._sleep)(match self._step {
			PomodoroStep::Work => self.work_duration,
			PomodoroStep::ShortBreak => self.short_break_duration,
			PomodoroStep::LongBreak => self.long_break_duration,
		});
	}

	pub fn get_step(&self) -> PomodoroStep {
		self._step
	}
}

fn main() {
	let (mut timer, canceller) = Timer::new2().unwrap();
	let sleeping = Arc::new(AtomicBool::new(false));
	let mut timer = PomodoroTimer::new(|duration| {
		sleeping.store(true, Ordering::Relaxed);
		_ = timer.sleep(duration);
		sleeping.store(false, Ordering::Relaxed);
	});

	{
		let sleeping = Arc::clone(&sleeping);
		ctrlc::set_handler(move || {
			if sleeping.load(Ordering::Relaxed) {
				_ = canceller.cancel();
			} else {
				exit(0);
			}
		})
		.unwrap();
	}

	let stdin = io::stdin();

	println!("Press Control+C at any time to skip the current timer.");

	loop {
		timer.next();

		match timer.get_step() {
			PomodoroStep::Work => println!("Time to work."),
			PomodoroStep::ShortBreak => println!("Take a short break."),
			PomodoroStep::LongBreak => println!("Take a long break."),
		}

		println!("Press enter to start timer or Control+C to exit.");

		let mut input = String::new();
		_ = stdin.read_line(&mut input);

		timer.sleep();
	}
}
