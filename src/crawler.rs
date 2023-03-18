// Standard library includes
use router::Router;
use std::collections::HashMap;
use std::fs;

// Library includes
//use iron::prelude::*;
//use iron::status;
//use mysql as my;
//use params::FromValue;
//use params::Params;
use serde_json::to_string_pretty;
use walkdir::{DirEntry, WalkDir};

use rocket::State;
use rocket::{get, put, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::okapi::schemars::{self, JsonSchema};
use rocket_okapi::openapi;
use rocket_okapi::openapi_get_routes_spec;
use rocket_okapi::settings::OpenApiSettings;
use serde::{Deserialize, Serialize};

// Local includes
use db;

pub fn get_photo_rand() -> (u64, String) {
	let pool = db::get_connection();

	// Select all photos from this source_id
	let mut connection = pool.get_conn().unwrap();
	let result = connection
		.query(
			r"
		SELECT photos.id as id, CONCAT(`full_path`,`relative_path`) as 
		`full_path` FROM `photos`, `sources`
		WHERE sources.id=photos.source
		ORDER BY RAND()
		LIMIT 1",
		)
		.unwrap();

	// Convert query resuts to HashMap
	let mut image = (0, String::from(""));
	result.for_each(|row| match row {
		Ok(row) => {
			let (id, full_path): (u64, String) = my::from_row(row);
			image = (id, full_path.to_string());
		}
		Err(_) => {
			image = (0, "".to_string());
		}
	});
	return image;
}
