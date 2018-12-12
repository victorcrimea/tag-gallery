// Stndard includes
use std::collections::HashMap;
use persistent::State;
use std::fs::File;
use std::io::Read;

// Library includes
use router::Router;
use iron::prelude::*;
use iron::status;
use mysql as my;
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

	let photo_id: u64 = id.parse::<u64>().unwrap_or(0);;

	// Check if photo exists
	let connection = db::get_connection();
	let result = connection.prep_exec(r"
	    SELECT id,
	           filesize,
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
						let size: u64 = row.take(1).unwrap_or(0);
						let latitude: f64 = row.take(2).unwrap_or(0.0);
						let longitude: f64 = row.take(3).unwrap_or(0.0);
						let altitude: f64 = row.take(4).unwrap_or(0.0);
						out_json = json!({
							"size" : size,
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