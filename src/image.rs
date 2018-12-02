// Stndard includes
use persistent::State;
use std::fs::File;
use std::io::Read;

// Library includes
use router::Router;
use iron::prelude::*;
use iron::status;
use mysql as my;
use persistent;

// Local includes
use db;
use Settings;

// This handler serves image of requested size
pub fn get(request: &mut Request) -> IronResult<Response> {
	let settings = request.get::<State<Settings>>().unwrap();
	//let gallery_folder = settings["gallery_folder"].as_str();
	let gallery_folder = "/storage/tag_gallery";
	let ref id = request.extensions.get::<Router>().unwrap()
	.find("id").unwrap_or("0");
	let ref size = request.extensions.get::<Router>().unwrap()
	.find("size").unwrap_or("0");

	// Check if photo exists
	let connection = db::get_connection();
	let result = connection.prep_exec(r"
	    SELECT photos.id FROM `photos`
	    WHERE photos.id = :id",
	params!{"id" => id});

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
    					let content_type = "image/jpeg".parse::<mime::Mime>().unwrap();
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

fn read_image(gallery_folder: &str, id: u64, size: &str) -> Option<Vec<u8>>{
	let mut buffer: Vec<u8> = vec![];
	let mut file = File::open(format!("{}/{}/{}.jpg", gallery_folder, size, id))
	.unwrap();

	match file.read_to_end(&mut buffer){
		Ok(_) => {
				return Some(buffer);
		},
		Err(_) => {
			return None;
		}
	}
}