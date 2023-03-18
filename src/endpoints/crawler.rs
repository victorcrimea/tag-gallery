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

mod helpers;
use helpers::crawl_source;
pub use helpers::get_photos;
use helpers::Photos;
use helpers::SourcePath;
use helpers::Sources;

pub fn get_routes_and_docs(settings: &OpenApiSettings) -> (Vec<rocket::Route>, OpenApi) {
	openapi_get_routes_spec![settings: add_source_path, list_source_paths, list_photos]
}

/// Adds source path to the database.
///
/// This function saves provided absolute path (on the server) to the database
/// and goes over all jpeg files recursively in order to add them to DB.

#[openapi(tag = "Crawler")]
#[post("/add_source_path?<path>")]
pub async fn add_source_path(mut db: Connection<MainDB>, path: &str) -> crate::ApiResult<u32> {
	let result = sqlx::query!(
		"
		INSERT INTO sources (full_path) VALUES (?)",
		path
	)
	.execute(&mut *db)
	.await?;

	let source_id = result.last_insert_id() as u32;

	crawl_source(&mut db, path.to_string(), &source_id).await?;

	sqlx::query!(
		"
		UPDATE sources
		SET   status = 'indexed'
		WHERE id = ?",
		source_id
	)
	.execute(&mut *db)
	.await?;
	Ok(Json(source_id))
}

/// Provides all available source paths
#[openapi(tag = "Crawler")]
#[get("/list_source_paths")]
pub async fn list_source_paths(mut db: Connection<MainDB>) -> crate::ApiResult<Sources> {
	let result = sqlx::query!(
		"
		SELECT sources.id,
	       full_path,
	       status,
	       COUNT(photos.id) as num_photos,
	       SUM(filesize) as size
		FROM sources
		LEFT JOIN photos on photos.source = sources.id
		GROUP BY sources.id",
	)
	.fetch_all(&mut *db)
	.await?;

	let paths: Vec<SourcePath> = result
		.into_iter()
		.map(|row| SourcePath {
			id: row.id,
			full_path: row.full_path,
			status: row.status,
			num_photos: row.num_photos as u32,
			size: row.size.unwrap().try_into().unwrap(),
		})
		.collect();

	Ok(Json(Sources { paths }))
}

#[openapi(tag = "Crawler")]
#[get("/list_photos/<source_id>")]
pub async fn list_photos(mut db: Connection<MainDB>, source_id: u32) -> crate::ApiResult<Photos> {
	let mut ids: Vec<u32> = vec![];

	for image in get_photos(&mut db, source_id).await? {
		ids.push(image.id);
	}

	Ok(Json(Photos { ids }))
}
