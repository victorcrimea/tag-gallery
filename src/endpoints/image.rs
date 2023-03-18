use rocket::State;
use rocket::{get, post, put, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use rocket_okapi::settings::OpenApiSettings;
use serde::{Deserialize, Serialize};

use crate::MainDB;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
    openapi_get_routes_spec![settings: get]
}

#[openapi(tag = "Image")]
#[get("/image/<id>/<size>")]
/// This handler serves image of requested size
pub fn get(mut db: Connection<MainDB>, id: u64, size: &str) -> crate::ApiResult<()> {
    // // Read global state
    // let rwlock = request.get::<State<Settings>>().unwrap();
    // let settings = rwlock.read().unwrap();
    // let gallery_folder = settings["gallery_folder"].as_str();

    // // Get url params
    // let ref id = request.extensions.get::<Router>().unwrap()
    // .find("id").unwrap_or("0");
    // let ref size = request.extensions.get::<Router>().unwrap()
    // .find("size").unwrap_or("0");

    // let photo_id: u64;
    // match *id {
    // 	"rand" => {
    // 		let photo = crawler::get_photo_rand();
    // 		photo_id = photo.0;
    // 	},
    // 	_ => {
    // 		photo_id = id.parse::<u64>().unwrap_or(0);;
    // 	}
    // }

    // // Check if photo exists
    // let connection = db::get_connection();
    // let result = connection.prep_exec(r"
    //     SELECT photos.id FROM `photos`
    //     WHERE photos.id = :id",
    // params!{"id" => photo_id});

    // match result {
    // 	Ok(result) => {
    // 		let mut id: u64 = 0;
    // 		for row in result {
    // 			id = my::from_row(row.unwrap());
    // 			break;
    // 		}

    // 		if id == 0 {
    // 			Ok(Response::with((status::NotFound, "")))
    // 		} else {
    // 			// Read image file and return
    // 			match read_image(gallery_folder, id, size) {
    // 				Some(data) => {
    // 					use iron::mime;
    //    					let content_type =
    //    						"image/jpeg".parse::<mime::Mime>().unwrap();
    // 					Ok(Response::with((content_type, status::Ok, data)))
    // 				},
    // 				None => {
    // 					Ok(Response::with((status::NotFound, "")))
    // 				},
    // 			}

    // 		}
    // 	},
    // 	Err(_) => {
    // 		Ok(Response::with((status::InternalServerError, "")))
    // 	}
    // }
    todo!()
}
