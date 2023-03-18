use exif::Exif;
use rocket::tokio::fs::File;
use rocket::tokio::stream;
use rocket::tokio::sync::mpsc;
use rocket::tokio::task;
use rocket::tokio::task::JoinHandle;
use rocket_db_pools::Connection;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use exif::{Reader, Tag, Value};

use crate::endpoints::crawler::get_photos;
use crate::MainDB;

#[derive(Debug)]
struct Job {
	source_id: u32,
}

#[derive(Debug)]
pub struct JobDone {
	source_id: u32,
}

pub struct Settings {
	gallery_folder: String,
}

#[derive(Debug)]
pub struct ImageProcessorPoolAsync {
	source_id: u32,
	thread: JoinHandle<()>,
	job_sender: mpsc::Sender<Job>,
	job_done_receiver: mpsc::Receiver<JobDone>,
	done_jobs: Vec<JobDone>,
}

impl ImageProcessorPoolAsync {
	/// Create a new ImageProcessorPool with only one working thread
	pub async fn new(
		db: &'static mut Connection<MainDB>,
		settings: Settings,
	) -> ImageProcessorPoolAsync {
		// Channel size = 0 means that there will be no bufferisation between
		// threads. So nothing should remain inside the sync_channel. Jobs will
		// go directly to the processing thread.
		let channel_size = 0;
		let (job_sender, mut jobreceiver) = mpsc::channel::<Job>(channel_size);
		let (job_done_sender, job_done_receiver) = mpsc::channel::<JobDone>(channel_size);
		let handle = task::spawn(async move {
			loop {
				// Waiting for job from the receiving end of the channel
				let job = jobreceiver.recv().await.unwrap();

				println!(
					"ImageProcessorPool got a job; Processing images \
					in source_id: {}",
					job.source_id
				);

				// Creating thumbnails for specified source
				match create_thumbs_in_source(db, settings.gallery_folder.clone(), job.source_id)
					.await
				{
					Ok(_) => {}
					Err(_) => {
						println!("Unable to process images in the source.");
					}
				}

				// Extracting EXIF data for specified source
				match process_exif(db, job.source_id).await {
					Ok(_) => {}
					Err(_) => {
						println!("Unable to extract EXIF data in the source.");
					}
				}

				// Set source_id status to resized
				// let connection = db::get_connection();
				// TODO: Implement saving to DB
				// let _result = connection.prep_exec(
				// 	r"
				//       UPDATE `sources`
				//       SET   `status` = 'resized'
				//       WHERE `id` = :source_id",
				// 	params! {"source_id" => job.source_id},
				// );

				// Preparing and sending JobDone object
				let job_done = JobDone {
					source_id: job.source_id,
				};
				println!("Job Done! source_id: {:?}", job.source_id);
				job_done_sender.send(job_done).await.unwrap();
			}
		});

		// Return newly created structure (object)
		ImageProcessorPoolAsync {
			source_id: 0,
			thread: handle,
			job_sender: job_sender,
			job_done_receiver: job_done_receiver,
			done_jobs: vec![],
		}
	}

	/// Add processing job into separate task
	pub async fn add_source_to_process(&self, source_id: u32) -> Result<bool, &'static str> {
		let job = Job {
			source_id: source_id,
		};

		match self.job_sender.send(job).await {
			Ok(_) => Ok(true),
			Err(_) => Err("Cannot send job to ImageProcessorPool"),
		}
	}

	// pub fn status_of(&mut self, source_id: u64) -> bool {
	// 	// Getting all JobDone's from channel
	// 	loop {
	// 		match self.job_done_receiver.lock().unwrap().try_recv() {
	// 			Ok(job_done) => {
	// 				self.done_jobs.push(job_done);
	// 			}
	// 			Err(_) => {
	// 				break;
	// 			}
	// 		}
	// 	}

	// 	//Searching for requested source_id
	// 	match self
	// 		.done_jobs
	// 		.iter()
	// 		.find(|&job_done| job_done.source_id == source_id)
	// 	{
	// 		Some(_) => true,
	// 		None => false,
	// 	}
	// }
}

/// Extracts GPS EXIF data from photos in source_id
async fn process_exif(db: &mut Connection<MainDB>, source_id: u32) -> Result<u32, bool> {
	println!("Extracting EXIF!");
	let images = get_photos(db, source_id).await.unwrap();
	for image in images.into_iter() {
		// Open file
		let mut file = File::open(&image.full_path).await.unwrap();
		let mut image_bytes = Vec::new();
		file.read_to_end(&mut image_bytes).await;
		//let bufreader = BufReader::new(&file).await;
		let exif = Reader::new().read_raw(image_bytes).unwrap();

		let latitude = read_latitude(&exif);
		let longitude = read_longitude(&exif);
		let altitude = read_altitude(&exif);
		let gps_date = read_gps_date(&exif);
		let gps_time = read_gps_time(&exif);
		let exif_datetime = read_exif_datetime(&exif);
		let exif_width = read_exif_width(&image.full_path).await;
		let exif_height = read_exif_height(&image.full_path).await;

		println!("EXIF processed");
		//let connection = db::get_connection();

		// Set image data
		// let _result = connection.prep_exec(
		// 	r"
		// 	     UPDATE `photos`
		// 	     SET   `exif_latitude` = :latitude,
		// 	           `exif_longitude` = :longitude,
		// 	           `exif_altitude`  = :altitude,
		// 	           `exif_gps_date`  = :gps_date,
		// 	           `exif_gps_time`  = :gps_time,
		// 	           `exif_datetime`  = :exif_datetime,
		// 	           `exif_width`     = :exif_width,
		// 	           `exif_height`    = :exif_height
		// 	     WHERE `id` = :id",
		// 	params! {
		// 		"id"            => id,
		// 		"latitude"      => latitude,
		// 		"longitude"     => longitude,
		// 		"altitude"      => altitude,
		// 		"gps_date"      => gps_date,
		// 		"gps_time"      => gps_time,
		// 		"exif_datetime" => exif_datetime,
		// 		"exif_width"    => exif_width,
		// 		"exif_height"   => exif_height
		// 	},
		// );

		//TODO: Implement quesry result check
	}
	Ok(0)
}

/// Gets GPS longitude in absolute floating-point value
/// like -3.001123. West longitudes represented as negative number.
///
/// # Arguments
/// * `reader` - EXIF Reader object from kamadak-exif library
fn read_longitude(exif: &Exif) -> f64 {
	let mut longitude: f64 = 0.0;

	// Longtitude numeriacal value
	if let Some(field) = exif.get_field(Tag::GPSLongitude, exif::In::PRIMARY) {
		match field.value {
			Value::Rational(ref vec) if !vec.is_empty() => {
				longitude = vec[0].to_f64() + vec[1].to_f64() / 60.0 + vec[2].to_f64() / 3600.0;
			}
			_ => {}
		}
	}

	// Longtitude reference East or West
	let longitude_ref;
	match exif.get_field(Tag::GPSLongitudeRef, exif::In::PRIMARY) {
		Some(field) => match format!("{}", field.value.display_as(field.tag)).as_str() {
			"E" => longitude_ref = "E",
			"W" => longitude_ref = "W",
			_ => longitude_ref = "E",
		},
		None => {
			longitude_ref = "E";
		}
	}
	if longitude_ref == "W" {
		longitude = longitude * -1.0;
	}
	return longitude;
}

/// Gets GPS latitude in absolute floating-point value
/// like -14.5463129. South latitudes represented as negative number.
///
/// # Arguments
/// * `reader` - EXIF Reader object from kamadak-exif library
fn read_latitude(exif: &Exif) -> f64 {
	let mut latitude: f64 = 0.0;

	// Latitude numeriacal value
	if let Some(field) = exif.get_field(Tag::GPSLatitude, exif::In::PRIMARY) {
		match field.value {
			Value::Rational(ref vec) if !vec.is_empty() => {
				latitude = vec[0].to_f64() + vec[1].to_f64() / 60.0 + vec[2].to_f64() / 3600.0;
				//println!("GPS latitude is {}", latitude);
			}
			_ => {}
		}
	}

	// Latitude reference North or South
	let latitude_ref;
	match exif.get_field(Tag::GPSLatitudeRef, exif::In::PRIMARY) {
		Some(field) => match format!("{}", field.value.display_as(field.tag)).as_str() {
			"N" => latitude_ref = "N",
			"S" => latitude_ref = "S",
			_ => latitude_ref = "N",
		},
		None => {
			latitude_ref = "N";
		}
	}
	if latitude_ref == "S" {
		latitude = latitude * -1.0;
	}
	return latitude;
}

/// Gets GPS altitude in absolute floating-point value
/// Altitudes below sea level represented as negative number.
///
/// # Arguments
/// * `reader` - EXIF Reader object from kamadak-exif library
fn read_altitude(exif: &Exif) -> f64 {
	let mut altitude: f64 = 0.0;
	// Altitude numeriacal value
	if let Some(field) = exif.get_field(Tag::GPSAltitude, exif::In::PRIMARY) {
		match field.value {
			Value::Rational(ref vec) if !vec.is_empty() => {
				altitude = vec[0].to_f64();
			}
			_ => {}
		}
	}

	// Altitude reference Above or Below (sea level)
	let altitude_ref;
	match exif.get_field(Tag::GPSAltitudeRef, exif::In::PRIMARY) {
		Some(field) => {
			//println!("{}", field.value.display_as(field.tag));
			match format!("{}", field.value.display_as(field.tag)).as_str() {
				"above sea level" => altitude_ref = "above sea level",
				"below sea level" => altitude_ref = "below sea level",
				_ => altitude_ref = "above sea level",
			}
		}
		None => {
			altitude_ref = "above sea level";
		}
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
fn read_gps_date(exif: &Exif) -> String {
	let iso_date_length = 10;
	let mut buf = String::with_capacity(iso_date_length);

	// Date as string (in ISO format)
	if let Some(field) = exif.get_field(Tag::GPSDateStamp, exif::In::PRIMARY) {
		buf = format!("{}", field.value.display_as(field.tag));
	}
	return buf;
}

/// Gets GPS time in ISO 8601 format
/// For example: "07:20:03"
///
/// # Arguments
/// * `reader` - EXIF Reader object from kamadak-exif library
fn read_gps_time(exif: &Exif) -> String {
	let iso_time_length = 8;
	let mut buf = String::with_capacity(iso_time_length);

	// Time as string (in ISO format)
	if let Some(field) = exif.get_field(Tag::GPSTimeStamp, exif::In::PRIMARY) {
		buf = format!("{}", field.value.display_as(field.tag));
	}
	return buf;
}

/// Gets EXIF time in ISO 8601 format
/// For example: "07:20:03"
///
/// # Arguments
/// * `reader` - EXIF Reader object from kamadak-exif library
fn read_exif_datetime(exif: &Exif) -> String {
	let iso_time_length = 8;
	let mut buf = String::with_capacity(iso_time_length);

	// Time as string (in ISO format)
	if let Some(field) = exif.get_field(Tag::DateTime, exif::In::PRIMARY) {
		buf = format!("{}", field.value.display_as(field.tag));
	}
	return buf;
}

async fn read_exif_width(full_path: &String) -> u32 {
	let output = Command::new("exiftool")
		.arg("-b")
		.arg("-ExifImageWidth")
		.arg(&full_path)
		.output()
		.await
		.expect("failed to execute process");

	String::from_utf8_lossy(&output.stdout)
		.to_string()
		.parse()
		.unwrap_or(0)
}

async fn read_exif_height(full_path: &String) -> u32 {
	let output = Command::new("exiftool")
		.arg("-b")
		.arg("-ExifImageHeight")
		.arg(&full_path)
		.output()
		.await
		.expect("failed to execute process");

	String::from_utf8_lossy(&output.stdout)
		.to_string()
		.parse()
		.unwrap_or(0)
}

/// Copies EXIF Orientatio tag from one image to another
///
/// # Arguments
/// * `src` - source image path
/// * `dst` - destination image path
async fn copy_exif_orientation(src: &str, dst: &str) {
	Command::new("exiftool")
		.arg("-TagsFromFile")
		.arg(src)
		.arg("-Orientation")
		.arg("-overwrite_original")
		.arg(dst)
		.output()
		.await
		.expect("failed to execute process");
}

/// Creates thumbnail images for corresponding source folder
async fn create_thumbs_in_source(
	db: &mut Connection<MainDB>,
	gallery_folder: String,
	source_id: u32,
) -> Result<u64, bool> {
	let images = get_photos(db, source_id).await.unwrap();

	for image in images.into_iter() {
		println!("Doing something for {:?}", image.full_path);

		// Create large image
		Command::new("convert")
			.arg(&image.full_path)
			.arg("-resize")
			.arg("1200x1200")
			.arg("-quality")
			.arg("75")
			.arg(format!("{}/large/{}.jpg", gallery_folder, image.id))
			.output()
			.await
			.expect("failed to execute process");

		// Instead of medium image we get JPEG thumbnail
		let medium = format!("{}/medium/{}.jpg", gallery_folder, image.id);
		match Command::new("exiftool")
			.arg("-b")
			.arg("-ThumbnailImage")
			.arg(&image.full_path)
			.output()
			.await
		{
			Ok(out) => {
				if out.stdout.len() > 0 {
					let mut file = File::create(&medium).await.unwrap();
					file.write_all(&out.stdout).await.unwrap();
				} else {
					Command::new("convert")
						.arg(&image.full_path)
						.arg("-resize")
						.arg("600x600")
						.arg("-quality")
						.arg("70")
						.arg(&medium)
						.output()
						.await
						.expect("failed to execute process");
				}
			}
			Err(_) => {}
		}

		// Preserve EXIF orientation flag
		copy_exif_orientation(&image.full_path, &medium);

		// Create small image (thumbnail)
		Command::new("convert")
			.arg(&image.full_path)
			.arg("-resize")
			.arg("200x200")
			.arg("-quality")
			.arg("50")
			.arg(format!("{}/small/{}.jpg", gallery_folder, image.id))
			.output()
			.await
			.expect("failed to execute process");
	}

	Ok(0)
}
