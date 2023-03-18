use std::fs;

use rocket_db_pools::Connection;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use crate::MainDB;

#[derive(Serialize, Deserialize, Debug)]
struct GalleryImage {
	source_path: String,
	relative_path: String,
	size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
	pub id: u32,
	pub full_path: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourcePath {
	pub id: u32,
	pub full_path: String,
	pub status: String,
	pub num_photos: u32,
	pub size: u64,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Sources {
	pub paths: Vec<SourcePath>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Photos {
	pub ids: Vec<u32>,
}

/// Extracts images from source
/// Goes recursively over all files in specified path and adds found jpegs to database
pub async fn crawl_source(
	db: &mut Connection<MainDB>,
	crawl_path: String,
	source_id: &u32,
) -> Result<bool, crate::error::ApiError> {
	let source_path = crawl_path.clone();
	let relative_paths = get_paths_of_images(crawl_path).await;

	let mut images: Vec<GalleryImage> = vec![];

	for rel_path in relative_paths.iter() {
		let full_path = format!("{}{}", source_path, rel_path);
		let metadata = fs::metadata(&full_path).unwrap();

		images.push(GalleryImage {
			source_path: source_path.clone(),
			relative_path: rel_path.clone(),
			size: metadata.len(),
		})

		//println!("{} - {} bytes", rel_path, metadata.len());
	}

	save_images_to_db(db, images, source_id).await
}

/// Extacts relative paths of images in specified directory recursively.
async fn get_paths_of_images(search_path: String) -> Vec<String> {
	let walker = WalkDir::new(search_path.clone()).into_iter();

	let mut paths: Vec<String> = vec![];

	for entry in walker.filter_map(|e| {
		if is_jpg(e.as_ref().unwrap()) {
			Some(e)
		} else {
			None
		}
	}) {
		let crawl_path_len = search_path.chars().count();
		let entry = entry.unwrap();
		let full_path = match entry.path().to_str() {
			Some(path) => path.to_string(),
			None => "".to_string(),
		};

		//paths.push(full_path.clone());

		let relative_path: String = full_path.chars().skip(crawl_path_len).collect();

		paths.push(relative_path);

		//println!("{:?}", paths);
	}
	paths
}

/// Checks if file is jpeg-related
///
/// It simply checks filename for the jp(e)g ending.
fn is_jpg(entry: &DirEntry) -> bool {
	entry
		.file_name()
		.to_str()
		.map(|s| {
			s.ends_with(".jpg")
				|| s.ends_with(".jpeg")
				|| s.ends_with(".JPG")
				|| s.ends_with(".JPEG")
		})
		.unwrap_or(false)
}

/// Adds images to database
///
/// It saves only meta information about images to database
async fn save_images_to_db(
	db: &mut Connection<MainDB>,
	images: Vec<GalleryImage>,
	source_id: &u32,
) -> Result<bool, crate::error::ApiError> {
	for image in images.iter() {
		sqlx::query!(
			"
			INSERT INTO photos
			        (relative_path, source, filesize)
			VALUES  (?, ?, ?)",
			image.relative_path.clone(),
			source_id,
			image.size
		)
		.execute(&mut **db)
		.await?;
	}
	Ok(true)
}

pub async fn get_photos(
	db: &mut Connection<MainDB>,
	source_id: u32,
) -> Result<Vec<Image>, crate::error::ApiError> {
	let result = sqlx::query!(
		"
		SELECT photos.id as id,
		       CONCAT(sources.full_path, photos.relative_path) as full_path
		FROM photos 
		    LEFT JOIN sources ON sources.id = photos.source
		WHERE sources.id=?
		",
		source_id
	)
	.fetch_all(&mut **db)
	.await?;

	let output: Vec<_> = result
		.into_iter()
		.map(|row| Image {
			id: row.id,
			full_path: row.full_path.unwrap(),
		})
		.collect();

	Ok(output)
}
