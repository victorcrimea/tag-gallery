// Standard library includes
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::collections::HashMap;
use std::sync::mpsc;
use std::process::Command;

// Library includes
use iron::typemap::Key;
use rayon::prelude::*;
use rayon::iter::IntoParallelIterator;
use mysql as my;

// Local includes
use db;

#[derive(Debug)]
pub struct ImageProcessorPool {
	source_id: u64,
	thread: JoinHandle<()>,
	job_sender: mpsc::SyncSender<Job>,
	pub job_done_receiver: Arc<Mutex<mpsc::Receiver<JobDone>>>,
	done_jobs: Vec<JobDone>
}

unsafe impl Sync for ImageProcessorPool {}

#[derive(Debug)]
struct Job{
	source_id: u64,
}

#[derive(Debug)]
pub struct JobDone {
	source_id: u64,
}


unsafe impl Sync for JobDone {}

impl ImageProcessorPool {
	/// Create a new ImageProcessorPool with only one working thread
	pub fn new(settings: HashMap<String, String>) -> ImageProcessorPool {

		// Channel size = 0 means that there will be no bufferisation between
		// threads. So nothing should remain inside the sync_channel. Jobs will
		// go directly to the processing thread.
		let channel_size = 0;
		let (job_sender, jobreceiver) = mpsc::sync_channel::<Job>(channel_size);
		let (job_done_sender, job_done_receiver) = mpsc::channel::<JobDone>();
		let thread = thread::spawn(move || {
			loop {
				// Waiting for job from the receiving end of the channel
				let job = jobreceiver.recv().unwrap();
				
				println!("ImageProcessorPool got a job; Processing images \
					in source_id: {}", job.source_id);
				// Creating thumbnails for specified source
				match ImageProcessorPool::create_thumbs_in_source(
					settings["gallery_folder"].clone(), job.source_id) {
					Ok(_) => {},
					Err(_) => {
						println!("Unable to process images in the source.");
					}
				}

				// Set source_id status to resized
				let connection = db::get_connection();
				 let _result = connection.prep_exec(r"
				      UPDATE `sources` 
				      SET   `status` = 'resized' 
				      WHERE `id` = :source_id", 
				      params!{"source_id" => job.source_id});

				// Preparing and sending JobDone object
				let job_done = JobDone {source_id: job.source_id};
				println!("Job Done! source_id: {:?}", job.source_id);
				job_done_sender.send(job_done).unwrap();
			}
			
		});

		// Return newly created structure (object)
		ImageProcessorPool{
			source_id: 0,
			thread: thread,
			job_sender: job_sender,
			job_done_receiver: Arc::new(Mutex::new(job_done_receiver)),
			done_jobs: vec![]
		}
	}

	/// Add processing task into separate thread
	pub fn add_source_to_process(&self, source_id: u64)
		-> Result<bool, &'static str>{
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
		// Getting all JobDone's from channel
		loop {
			match self.job_done_receiver.lock().unwrap().try_recv() {
				Ok(job_done) => {
					self.done_jobs.push(job_done);
				},
				Err(_) => {
					break;
				}
			}
		}
		
		//Searching for requested source_id
		match self.done_jobs
			.iter()
			.find(|&job_done| job_done.source_id == source_id) {
			Some(_) => true,
			None => false
		}
	}

	/// Creates thumbnail images for corresponding source folder
	fn create_thumbs_in_source(gallery_folder: String, source_id: u64)
		-> Result<u64, bool> {

		let connection = db::get_connection();
		
		// Select all photos from this source_id
		let result = connection.prep_exec(r"
			SELECT photos.id as id, CONCAT(`full_path`,`relative_path`) as 
			`full_path` FROM `photos`, `sources`
			WHERE sources.id=photos.source AND
			sources.id=:source_id",
			params!{"source_id" => source_id}
			).unwrap();
		
		// We'll store images as pair id - absolute path
		let mut images: HashMap<u64, String> = HashMap::new();

		// Convert query resuts to HashMap
		result.for_each(|row| {
			match row {
				Ok(row) => {
					let (id, full_path): (u64, String) = my::from_row(row);
					images.insert(id, full_path);
				},
				Err(_) => {}
			}
		});

		images.into_par_iter().for_each(|(id, full_path)| {
			println!("Doing something for {:?}", full_path);

			// Create large image
			Command::new("convert")
				.arg(&full_path)
				.arg("-resize")
				.arg("1200x1200")
				.arg("-quality")
				.arg("100")
				.arg(format!("{}/large/{}.jpg", gallery_folder, id))
				.output()
				.expect("failed to execute process");

			// Create medium image
			Command::new("convert")
				.arg(&full_path)
				.arg("-resize")
				.arg("800x800")
				.arg("-quality")
				.arg("100")
				.arg(format!("{}/medium/{}.jpg", gallery_folder, id))
				.output()
				.expect("failed to execute process");

			// Create small image (thumbnail)
			Command::new("convert")
				.arg(&full_path)
				.arg("-resize")
				.arg("160x160")
				.arg("-quality")
				.arg("100")
				.arg(format!("{}/small/{}.jpg", gallery_folder, id))
				.output()
				.expect("failed to execute process");
		});

		Ok(0)
	}
}

/// Used as a key to reference the ImageProcessorPool
pub struct ImageProcessorPoolShared;
impl Key for ImageProcessorPoolShared { type Value = ImageProcessorPool; }