use std::thread;
use std::thread::JoinHandle;
use std::collections::HashMap;
use iron::typemap::Key;
use std::sync::mpsc;
use image;
use image::{GenericImage};

use db;
use mysql as my;

#[derive(Debug)]
pub struct ImageProcessorPool {
	source_id: u64,
	thread: JoinHandle<()>,
	job_sender: mpsc::SyncSender<Job>,
	job_done_receiver: mpsc::Receiver<JobDone>,
	done_jobs: Vec<JobDone>
}

unsafe impl Sync for ImageProcessorPool {}

#[derive(Debug)]
struct Job{
	source_id: u64
}

#[derive(Debug)]
struct JobDone {
	source_id: u64
}

impl ImageProcessorPool {
	/// Create a new ImageProcessorPool with only one working thread
	pub fn new() -> ImageProcessorPool {

		//Channel size = 0 means that there will be no bufferisation between threads
		let channel_size = 0;
		let (job_sender, jobreceiver) = mpsc::sync_channel::<Job>(channel_size);
		let (job_done_sender, job_done_receiver) = mpsc::channel::<JobDone>();

		let thread = thread::spawn(move || {
			loop {
				let job = jobreceiver.recv().unwrap();
				
				println!("ImageProcessorPool got a job; Processing images in source_id: {}", job.source_id);
				ImageProcessorPool::create_thumbs_in_source(job.source_id);

				let job_done = JobDone {source_id: job.source_id};
				println!("Job Done! source_id: {:?}", job.source_id);
				job_done_sender.send(job_done).unwrap();
			}
			
		});

		ImageProcessorPool{
			source_id: 0,
			thread: thread,
			job_sender: job_sender,
			job_done_receiver: job_done_receiver,
			done_jobs: vec![]
		}
	}

	/// Add processing task into separate thread
	pub fn add_source_to_process(&self, source_id: u64) -> Result<bool, &'static str>{
		let job = Job {source_id: source_id};
		
		match self.job_sender.try_send(job) {
			Ok(_) => Ok(true),
			Err(_) => Err("Cannot send job to ImageProcessorPool")
		}
	}


	/// Returns status of requested job
	/// job is determined by source_id
	///
	/// # Arguments
	/// * `source_id` - db identifier of source_path to get status of
	/// 
	/// # Example
	///
	/// ```
	/// // You can have rust code between fences inside the comments
	/// // If you pass --test to Rustdoc, it will even test it for you!
	/// let task_status = image_processing_pool::status_of(3);
	/// ```
	pub fn status_of(&mut self, source_id: u64) -> bool {
		//Getting all JobDone's from channel
		loop {
			match self.job_done_receiver.try_recv() {
				Ok(job_done) => {
					self.done_jobs.push(job_done);
				},
				Err(try_recv_error) => {
					break;
				}
			}
		}
		
		//Searching for requested source_id
		match self.done_jobs.iter().find(|&job_done| job_done.source_id == source_id) {
			Some(value) => true,
			None => false
		}
	}

	fn create_thumbs_in_source(source_id: u64) -> Result<u64, bool> {
		let connection = db::get_connection();
		let result = connection.prep_exec(r"
			SELECT `id`,`relative_path` FROM `photos`
			WHERE `source`=:source_id",
			params!{"source_id" => source_id}
			).unwrap();
		
		let mut images: HashMap<u64, String> = HashMap::new();

		result.for_each(|row| {
			match row {
				Ok(row) => {
					let (id, relative_path) = my::from_row(row);
					images.insert(id, relative_path);
				},
				Err(_) => {}
			}
		});

		Ok(0)
	}

	fn create_thumbnail() -> Result<bool, bool> {

		Ok(true)
	}
}

pub struct ImageProcessorPoolShared;
impl Key for ImageProcessorPoolShared { type Value = ImageProcessorPool; }