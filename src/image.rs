// Stndard includes

use std::collections::HashMap;
use persistent::State;
use std::fs::File;
use std::io::Read;


// Library includes
use router::Router;
use iron::prelude::*;
use iron::status;
use params::Params;
use params::FromValue;
use mysql as my;
use mysql::chrono::{NaiveDate,NaiveTime,NaiveDateTime};

use serde_json::to_string_pretty;

// Local includes
use db;
use Settings;
use crawler;

// This handler serves image of requested size
pub fn get(request: &mut Request) -> IronResult<Response> {
	// Read global state
	let rwlock = request.get::<State<Settings>>().unwrap();
	let settings = rwlock.read().unwrap();
	let gallery_folder = settings["gallery_folder"].as_str();
	
	// Get url params
	let ref id = request.extensions.get::<Router>().unwrap()
	.find("id").unwrap_or("0");
	let ref size = request.extensions.get::<Router>().unwrap()
	.find("size").unwrap_or("0");

	let photo_id: u64;
	match *id {
		"rand" => {
			let photo = crawler::get_photo_rand();
			photo_id = photo.0;
		},
		_ => {
			photo_id = id.parse::<u64>().unwrap_or(0);;
		}
	}

	// Check if photo exists
	let connection = db::get_connection();
	let result = connection.prep_exec(r"
	    SELECT photos.id FROM `photos`
	    WHERE photos.id = :id",
	params!{"id" => photo_id});

	match result {
		Ok(result) => {
			let mut id: u64 = 0;
			for row in result {
				id = my::from_row(row.unwrap());
				break;
			}

			if id == 0 {
				Ok(Response::with((status::NotFound, "")))
			} else {
				// Read image file and return
				match read_image(gallery_folder, id, size) {
					Some(data) => {
						use iron::mime;
    					let content_type =
    						"image/jpeg".parse::<mime::Mime>().unwrap();
						Ok(Response::with((content_type, status::Ok, data)))
					},
					None => {
						Ok(Response::with((status::NotFound, "")))
					},
				}
				
			}
		},
		Err(_) => {
			Ok(Response::with((status::InternalServerError, "")))
		}
	}
		
}

pub fn info(request: &mut Request) -> IronResult<Response> {
	// Get url params
	let ref id = request.extensions.get::<Router>().unwrap()
	.find("id").unwrap_or("0");

	let photo_id: u64 = id.parse::<u64>().unwrap_or(0);

	// Check if photo exists
	let connection = db::get_connection();
	let result = connection.prep_exec(r"
	    SELECT id,
	           filesize,
	           exif_gps_date,
	           exif_gps_time,
	           UNIX_TIMESTAMP(exif_datetime) as `exif_unix_timestamp`,
	           exif_latitude,
	           exif_longitude,
	           exif_altitude
	    FROM `photos`
	    WHERE photos.id = :id",
	params!{"id" => photo_id});

	match result {
		Ok(result) => {
			// We get here only if image exists in DB
			let mut out_json = json!({});
			result.for_each(|row|{
				match row {
					Ok(mut row) => {
						let size: u64 = row.take("size").unwrap_or(0);
						let gps_date: NaiveDate = row.take("exif_gps_date").unwrap_or(NaiveDate::from_ymd(1970,1,1));
						let gps_date: String = gps_date.to_string();
						let gps_time: NaiveTime = row.take("exif_gps_time").unwrap_or(NaiveTime::from_hms(0,0,0));
						let gps_time: String = gps_time.to_string();
						let exif_timestamp: i64 = row.take("exif_unix_timestamp").unwrap_or(0);
						//let exif_timestamp: String = exif_timestamp.to_string();
						let latitude: f64 = row.take("exif_latitude").unwrap_or(0.0);
						let longitude: f64 = row.take("exif_longitude").unwrap_or(0.0);
						let altitude: f64 = row.take("exif_altitude").unwrap_or(0.0);
						out_json = json!({
							"size" : size,
							"gps_date" : gps_date,
							"gps_time" : gps_time,
							"exif_timestamp" : exif_timestamp,
							"latitude": latitude,
							"longitude": longitude,
							"altitude": altitude
						});
					},
					Err(_) => {}
				}
			});

			Ok(Response::with(
				(status::Ok, to_string_pretty(&out_json).unwrap())
			))
		},
		Err(_) => {
			Ok(Response::with((status::NotFound, "")))
		}
	}
		
}

fn read_image(gallery_folder: &str, id: u64, size: &str) -> Option<Vec<u8>>{
	let mut buffer: Vec<u8> = vec![];
	let file = File::open(
		format!("{}/{}/{}.jpg", gallery_folder, size, id));
	
	match file {
		Ok(mut file) => {
			match file.read_to_end(&mut buffer){
				Ok(_) => {
						return Some(buffer);
				},
				Err(_) => {
					return None;
				}
			}
		},
		Err(_) => {
			return None
		}
	}
}

pub fn add_tag(request: &mut Request) -> IronResult<Response> {
	// Get url params
	let ref id = request.extensions.get::<Router>().unwrap()
	.find("id").unwrap_or("0");
	
	let photo_id: u64 = id.parse::<u64>().unwrap_or(0);

	// Get form data
	let params = request.get::<Params>().unwrap();
	let tag = String::from_value(&params["tag"]).unwrap();

	//Add this tag if not added yet
	let connection = db::get_connection();
	let result = connection.prep_exec(r"
	    INSERT INTO `tags`
	           (`tag`   , `creation_date`)
	    VALUES (:tag, now())",
	params!{"tag" => tag.clone()});

	match result {
		Ok(result) => {
			// Tag was added to db
			//Ok(Response::with((status::Ok, "REMOVE1")));
		},
		Err(result) => {
			// Tag is already exists
			//Ok(Response::with((status::Ok, "REMOVE2")));
		}
	}

	//Ok(Response::with((status::Ok, result.warnings().to_string())))

	// Attach this tag to the photo if not attached yet
	let result = connection.prep_exec(r"
	    INSERT INTO `photos_tags`
	           (`photo_id`, `tag`, `type`,   `creation_date`)
	    VALUES (:photo_id, :tag,  'manual', now())",
	params!{"photo_id" => photo_id, "tag" => tag});

	
	match result {
		Ok(result) => {
			// Tag was added to image
			Ok(Response::with((status::Ok, "Tag added")))
		},
		Err(result) => {
			// Tag is already added to this image
			Ok(Response::with((status::Ok, "Tag already there")))
		}
	}

}