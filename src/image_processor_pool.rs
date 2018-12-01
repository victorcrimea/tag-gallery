// Standard library includes
use std::sync::Mutex;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::collections::HashMap;
use std::sync::mpsc;
use std::process::Command;
use std::fs::File;
use std::io::BufReader;
use std::string::String;

// Library includes
use iron::typemap::Key;
use rayon::prelude::*;
use rayon::iter::IntoParallelIterator;
use mysql as my;
use exif::{Reader, Value, Tag};

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

				// Extracting EXIF location data for specified source
				match ImageProcessorPool::process_gps(job.source_id){
					Ok(_) => {},
					Err(_) => {
						println!("Unable to extract EXIF data in the source.");
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


	/// Gets GPS latitude in absolute floating-point value
	/// like -14.5463129. South latitudes represented as negative number.
	///
	/// # Arguments 
	/// * `reader` - EXIF Reader object from kamadak-exif library
	fn read_latitude(reader: &Reader) -> f64 {
		let mut latitude: f64 = 0.0;

		// Latitude numeriacal value
		if let Some(field) = reader.get_field(Tag::GPSLatitude, false) {
			match field.value {
				Value::Rational(ref vec) if !vec.is_empty() => {
						latitude = vec[0].to_f64() + 
						vec[1].to_f64()/60.0 +
						vec[2].to_f64()/3600.0;
						//println!("GPS latitude is {}", latitude);
					},
				_ => {},
			}
		}

		// Latitude reference North or South
		let latitude_ref;
		match reader.get_field(Tag::GPSLatitudeRef, false) {
			Some(field) => {
				match format!("{}", field.value.display_as(field.tag))
				.as_str(){
					"N" => latitude_ref = "N",
					"S" => latitude_ref = "S",
					_   => latitude_ref = "N",
				}
			},
			None => {
				latitude_ref = "N";
			},
		}
		if latitude_ref == "S" {
			latitude = latitude * -1.0;
		}
		return latitude;
	}

	/// Gets GPS longitude in absolute floating-point value
	/// like -3.001123. West longitudes represented as negative number.
	///
	/// # Arguments 
	/// * `reader` - EXIF Reader object from kamadak-exif library
	fn read_longitude(reader: &Reader) -> f64 {
		let mut longitude: f64 = 0.0;

		// Longtitude numeriacal value
		if let Some(field) = reader.get_field(Tag::GPSLongitude, false) {
			match field.value {
				Value::Rational(ref vec) if !vec.is_empty() => {
						longitude = vec[0].to_f64() + 
						vec[1].to_f64()/60.0 +
						vec[2].to_f64()/3600.0;
					},
				_ => {},
			}
		}

		// Longtitude reference East or West
		let longitude_ref;
		match reader.get_field(Tag::GPSLongitudeRef, false) {
			Some(field) => {
				match format!("{}", field.value.display_as(field.tag))
				.as_str(){
					"E" => longitude_ref = "E",
					"W" => longitude_ref = "W",
					_   => longitude_ref = "E",
				}
			},
			None => {
				longitude_ref = "E";
			},
		}
		if longitude_ref == "W" {
			longitude = longitude * -1.0;
		}
		return longitude;
	}

	/// Gets GPS altitude in absolute floating-point value
	/// Altitudes below sea level represented as negative number.
	///
	/// # Arguments 
	/// * `reader` - EXIF Reader object from kamadak-exif library
	fn read_altitude(reader: &Reader) -> f64 {
		let mut altitude: f64 = 0.0;
		// Altitude numeriacal value
		if let Some(field) = reader.get_field(Tag::GPSAltitude, false) {
			match field.value {
				Value::Rational(ref vec) if !vec.is_empty() => {
						altitude = vec[0].to_f64();
				},
				_ => {},
			}
		}

		// Altitude reference Above or Below (sea level)
		let altitude_ref;
		match reader.get_field(Tag::GPSAltitudeRef, false) {
			Some(field) => {
				//println!("{}", field.value.display_as(field.tag));
				match format!("{}", field.value.display_as(field.tag))
				.as_str(){
					"above sea level" => altitude_ref = "above sea level",
					"below sea level" => altitude_ref = "below sea level",
					_   => altitude_ref = "above sea level",
				}
			},
			None => {
				altitude_ref = "above sea level";
			},
		}
		if altitude_ref == "below sea level" {
			altitude = altitude * -1.0;
		}

		return altitude;
	}

	/// Gets GPS date in ISO 8601 format
	/// For example: "2023-04-16"
	///
	/// # Arguments 
	/// * `reader` - EXIF Reader object from kamadak-exif library
	fn read_gps_date(reader: &Reader) -> String {
		let iso_date_length = 10;
		let mut buf = String::with_capacity(iso_date_length);


		// Date as string (in ISO format)
		if let Some(field) = reader.get_field(Tag::GPSDateStamp, false) {
			buf = format!("{}", field.value.display_as(field.tag));
		}
		return buf;
	}

	/// Gets GPS time in ISO 8601 format
	/// For example: "07:20:03"
	///
	/// # Arguments 
	/// * `reader` - EXIF Reader object from kamadak-exif library
	fn read_gps_time(reader: &Reader) -> String {
		let iso_time_length = 8;
		let mut buf = String::with_capacity(iso_time_length);

		// Time as string (in ISO format)
		if let Some(field) = reader.get_field(Tag::GPSTimeStamp, false) {
			buf = format!("{}", field.value.display_as(field.tag));
		}
		return buf;
	}


	/// Extracts GPS EXIF data from photos in source_id
	fn process_gps(source_id: u64) -> Result<u64, bool> {
		println!("Extracting EXIF!");
		let images = ImageProcessorPool::get_photos(source_id);
		images.into_iter().for_each(|(id, full_path)|{
			// Open file
			let file = File::open(&full_path).unwrap();
			let reader = Reader::new(&mut BufReader::new(&file)).unwrap();

			let latitude = ImageProcessorPool::read_latitude(&reader);
			let longitude = ImageProcessorPool::read_longitude(&reader);
			let altitude = ImageProcessorPool::read_altitude(&reader);
			let date = ImageProcessorPool::read_gps_date(&reader);
			let time = ImageProcessorPool::read_gps_time(&reader);
			let connection = db::get_connection();

			// Set image data
			let _result = connection.prep_exec(r"
			     UPDATE `photos` 
			     SET   `exif_latitude` = :latitude,
			           `exif_longitude` = :longitude,
			           `exif_altitude`  = :altitude,
			           `exif_gps_date`  = :date,
			           `exif_gps_time`  = :time 
			     WHERE `id` = :id", 
			params!{
				"id" => id,
				"latitude" => latitude,
				"longitude" => longitude,
				"altitude" => altitude,
				"date" => date,
				"time" => time
			});

			//TODO: Implement quesry result check
		});
		Ok(0)
	}

	fn get_photos(source_id: u64) -> HashMap<u64, String> {
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
		println!("images list size: {:?}", images.len());
		return images;
	}

	/// Creates thumbnail images for corresponding source folder
	fn create_thumbs_in_source(gallery_folder: String, source_id: u64)
		-> Result<u64, bool> {

		let images = ImageProcessorPool::get_photos(source_id);



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
			// Command::new("convert")
			// 	.arg(&full_path)
			// 	.arg("-resize")
			// 	.arg("800x800")
			// 	.arg("-quality")
			// 	.arg("100")
			// 	.arg(format!("{}/medium/{}.jpg", gallery_folder, id))
			// 	.output()
			// 	.expect("failed to execute process");

			// Instead of medium image we get JPEG thumbnail
			Command::new("exiftool")
				.arg("-b")
				.arg("-ThumbnailImage")
				.arg(&full_path)
				.arg("-o")
				.arg(format!("{}/medium/{}.jpg", gallery_folder, id))
				.output()
				.expect("failed to execute process");

			// Create small image (thumbnail)
			// Command::new("convert")
			// 	.arg(&full_path)
			// 	.arg("-resize")
			// 	.arg("160x160")
			// 	.arg("-quality")
			// 	.arg("100")
			// 	.arg(format!("{}/small/{}.jpg", gallery_folder, id))
			// 	.output()
			// 	.expect("failed to execute process");
		});

		Ok(0)
	}
}

/// Used as a key to reference the ImageProcessorPool
pub struct ImageProcessorPoolShared;
impl Key for ImageProcessorPoolShared { type Value = ImageProcessorPool; }