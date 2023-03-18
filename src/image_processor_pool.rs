// Standard library includes
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::process::Command;
use std::string::String;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;

// Library includes
use exif::{Reader, Tag, Value};
use rayon::iter::IntoParallelIterator;
use rayon::prelude::*;

// Local includes

#[derive(Debug)]
pub struct ImageProcessorPool {
	source_id: u64,
	thread: JoinHandle<()>,
	job_sender: mpsc::SyncSender<Job>,
	pub job_done_receiver: Arc<Mutex<mpsc::Receiver<JobDone>>>,
	done_jobs: Vec<JobDone>,
}

unsafe impl Sync for ImageProcessorPool {}

#[derive(Debug)]
struct Job {
	source_id: u64,
}

#[derive(Debug)]
pub struct JobDone {
	source_id: u64,
}

unsafe impl Sync for JobDone {}

impl ImageProcessorPool {}

/// Used as a key to reference the ImageProcessorPool
pub struct ImageProcessorPoolShared;
impl Key for ImageProcessorPoolShared {
	type Value = ImageProcessorPool;
}
