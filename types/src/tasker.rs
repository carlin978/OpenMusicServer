use crossbeam_deque::{Steal, Worker};
use std::{
	any::Any,
	collections::HashMap,
	panic::{UnwindSafe, catch_unwind},
	sync::{LazyLock, Mutex, mpsc},
	thread::{self, JoinHandle},
};

type TaskClosure = Box<dyn FnOnce() -> () + Send + Sync + UnwindSafe>;

struct Task {
	pub id: u32,
	pub closure: TaskClosure,
}
impl Task {
	const ID_ITER: LazyLock<Mutex<std::ops::RangeFrom<u32>>> =
		LazyLock::new(|| Mutex::new((0..).into_iter()));

	fn new(closure: TaskClosure) -> Self {
		Self {
			id: Self::ID_ITER
				.lock()
				.unwrap()
				.next()
				.expect("Somehow too many tasks were scheduled"),
			closure,
		}
	}
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
	Scheduled,
	Running,
	Success,
	Failure,
	NotFound,
}

enum TaskResult {
	Success(u32),
	Failure(u32),
}

pub struct ThreadedTaskRunner {
	worker_queue: Worker<Task>,
	thread_handle: JoinHandle<()>,
	task_status: HashMap<u32, TaskStatus>,
	result_recv: mpsc::Receiver<TaskResult>,
	killswitch: mpsc::Sender<()>,
}
impl ThreadedTaskRunner {
	pub fn new() -> Self {
		let worker_queue: Worker<Task> = Worker::new_fifo();

		let queue_stealer = worker_queue.stealer();

		let (result_sender, result_recv) = mpsc::channel::<TaskResult>();

		let (killswitch_sender, killswitch_recv) = mpsc::channel::<()>();

		let thread_handle = thread::spawn(move || {
			while killswitch_recv.try_recv().is_err() {
				match queue_stealer.steal() {
					Steal::Empty | Steal::Retry => (),
					Steal::Success(task) => match catch_unwind(task.closure) {
						Ok(_) => result_sender.send(TaskResult::Success(task.id)).unwrap(),
						Err(err) => {
							result_sender.send(TaskResult::Failure(task.id)).unwrap();
							eprintln!("Task #{} failed: {:#?}", task.id, err);
						}
					},
				}
			}
		});

		Self {
			worker_queue,
			thread_handle,
			task_status: Default::default(),
			result_recv,
			killswitch: killswitch_sender,
		}
	}

	pub fn schedule_task(&mut self, closure: TaskClosure) {
		let task = Task::new(closure);

		self.task_status.insert(task.id, TaskStatus::Scheduled);
		self.worker_queue.push(task);
	}

	pub fn cleanup_task(&mut self, id: u32) {
		self.task_status.remove(&id);
	}

	pub fn get_task_status(&mut self, id: u32) -> TaskStatus {
		while let Ok(task_result) = self.result_recv.try_recv() {
			match task_result {
				TaskResult::Success(id) => {
					self.task_status.insert(id, TaskStatus::Success);
				}
				TaskResult::Failure(id) => {
					self.task_status.insert(id, TaskStatus::Failure);
				}
			}
		}

		self.task_status
			.get(&id)
			.cloned()
			.unwrap_or(TaskStatus::NotFound)
	}

	pub fn join(self) -> thread::Result<()> {
		self.killswitch.send(()).ok();
		self.thread_handle.join()
	}
}

#[cfg(test)]
mod test {
	use std::{thread, time::Duration};

	#[test]
	fn tasker() {
		let mut tasker = super::ThreadedTaskRunner::new();

		for i in 1..=10 {
			tasker.schedule_task(Box::new(move || println!("Task #{}", i)));
		}

		thread::sleep(Duration::from_millis(50));

		tasker.join().unwrap()
	}
}
