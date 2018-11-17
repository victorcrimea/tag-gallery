use iron::prelude::*;
use iron::status;
use persistent::State;
use params::Params;
use params::FromValue;

use std::str::FromStr;
use serde_json::to_string_pretty;

use image_processor_pool::ImageProcessorPoolShared;

/// Creates thumbnails for images in source_path
/// This handler accepts source_path_id and starts thread 
/// that goes over all images inside this path
/// and creates thumbnails for them
pub fn process_source_path(request: &mut Request) -> IronResult<Response> {

	let params = request.get::<Params>().unwrap();
	let source_id: u64 = u64::from_str(
		String::from_value(&params["source_id"])
		.unwrap()
		.as_str()
		).unwrap_or(0);


	let rwlock = request.get::<State<ImageProcessorPoolShared>>().unwrap();
	let image_processor_pool = rwlock.write().unwrap();

	match image_processor_pool.add_source_to_process(source_id) {
		Ok(_) => {
			let out_json = json!({
				"status": "accepted",
			});
			Ok(
				Response::with(
					(status::Accepted, to_string_pretty(&out_json).unwrap())
				)
			)
		},
		Err(_) => {
			let out_json = json!({
				"status": "locked",
				"hint": "try later"
			});

			Ok(
				Response::with(
					(status::Locked, to_string_pretty(&out_json).unwrap())
				)
			)
		}
	}

}

pub fn process_status(request: &mut Request) -> IronResult<Response> {
	let params = request.get::<Params>().unwrap();
	let source_id: u64 = match params.find(&["source_id"]) {
		Some(_) => {
			u64::from_str(
				String::from_value(&params["source_id"])
				.unwrap_or(String::new())
				.as_str()
			).unwrap_or(0)
		},
		None => {
			0
		}
	};


	match source_id {
		0 => {
			return Ok(
				Response::with(
					(status::BadRequest, "source_id should be set")
				)
			);
		},
		_  => ()
	};


	let rwlock = request.get::<State<ImageProcessorPoolShared>>().unwrap();
	let mut image_processor_pool = rwlock.write().unwrap();

	//println!("{:?}", image_processor_pool);

	match image_processor_pool.status_of(source_id) {
		true => {
			let out_json = json!({
				"status": "done"
			});

			Ok(
				Response::with(
					(status::Ok, to_string_pretty(&out_json).unwrap())
				)
			)
		},
		false => {
			let out_json = json!({
				"status": "unknown",
				"hint": "Status not found. Maybe it's not ready yet"
			});

			Ok(
				Response::with(
					(status::NotFound, to_string_pretty(&out_json).unwrap())
				)
			)
		}
	}


	
}