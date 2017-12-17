use iron::prelude::*;
use iron::status;
use params::Params;
use params::FromValue;
use db;
use mysql as my;
use walkdir::{DirEntry, WalkDir};
use rayon::prelude::*;
use std::path::Path;
use std::fs;
use image;
use image::{GenericImage};


use serde_json::to_string_pretty;

#[derive(Serialize, Deserialize)]
struct SourcePath {
	id: u32,
	full_path: String,
	indexed: u32
}

#[derive(Serialize, Deserialize, Debug)]
struct GalleryImage {
	source_path: String,
	relative_path: String,
	size: u64
}

/// Provides all available source paths
pub fn list_source_paths(_request: &mut Request) -> IronResult<Response> {
	

	let connection = db::get_connection();
	let result = connection.prep_exec(r"SELECT * FROM `sources`", ()).unwrap();

	let mut paths: Vec<SourcePath> = vec![];


	result.for_each(|row| {
		match row {
			Ok(row) => {
				let (id, full_path, indexed) = my::from_row(row);
				paths.push(SourcePath{
					id: id,
					full_path: full_path,
					indexed: indexed
				});
			},
			Err(_) => {}
		}
	});

	let out_json = json!({
		"paths": paths,
	});

	Ok(
		Response::with(
			(status::Ok, to_string_pretty(&out_json).unwrap())
		)
	)
}


pub fn add_source_path(request: &mut Request) -> IronResult<Response> {
	/// Adds source path to the database
	let params = request.get_ref::<Params>().unwrap();

	let path = &params["path"];

	let connection = db::get_connection();
	let result = connection.prep_exec(r"
	     INSERT INTO `sources` 
	             (`full_path`) 
	     VALUES  (:path)", 
	     params!{"path" => String::from_value(path)});

	let mut source_id: u64 = 0;
	match result {
		Ok(result) => {
			source_id = result.last_insert_id();
		},
		Err(_) => ()
	}

	crawl_source(String::from_value(path).unwrap(), source_id);

	Ok(Response::with((status::Ok, "ok")))
}

fn is_jpg(entry: &DirEntry) -> bool {
	//println!("DEBUG: {:?}", entry.file_name().to_str());
	entry.file_name()
		.to_str()
		.map(|s| {
			s.ends_with(".jpg") || 
			s.ends_with(".jpeg") ||
			s.ends_with(".JPG") ||
			s.ends_with(".JPEG")
		})
		.unwrap_or(false)
}

fn crawl_source(crawl_path: String, source_id: u64) {
	/// Goes recursively over all files in specified path and adds found jpegs to database

	let source_path = crawl_path.clone();
	let relative_paths = get_paths_of_images(crawl_path);

	let mut images: Vec<GalleryImage> = vec![];

	for rel_path in relative_paths.iter(){
		let full_path = format!("{}{}", source_path, rel_path);
		let metadata = fs::metadata(&full_path).unwrap();

		images.push(
			GalleryImage{
				source_path: source_path.clone(),
				relative_path: rel_path.clone(),
				size: metadata.len()
			}
		)

		//println!("{} - {} bytes", rel_path, metadata.len());
	}
	
	save_images_to_db(images, source_id);
}

fn save_images_to_db(images: Vec<GalleryImage>, source_id: u64) -> Result<bool, my::MySqlError> {
	let connection = db::get_connection();
	
	for image in images.iter() {
		let result = connection.prep_exec(r"
		     INSERT INTO `photos` 
		             (`relative_path`, `source`, `filesize`) 
		     VALUES  (:relative_path, :source, :filesize)", 
			params!{
				"relative_path" => image.relative_path.clone(),
				"source" => source_id,
				"filesize" => image.size
			}
		);

		match result {
			Ok(result) => (),
			Err(e) => println!("{:?}", e)
		}

	}
	Ok(true)
}


fn get_paths_of_images(search_path: String) -> Vec<String> {
	let walker = WalkDir::new(search_path.clone()).into_iter();

	let mut paths: Vec<String>= vec![];

	for entry in walker.filter_map(|e| {
		if is_jpg(e.as_ref().unwrap()) {
			Some(e)
		} else {
			None
		}
	}) {
		let crawl_path_len = search_path.chars().count();
		let entry = entry.unwrap();
		let full_path = match entry.path().to_str(){ 
			Some(path) => path.to_string(),
			None       => "".to_string()
		};
		
		//paths.push(full_path.clone());

		let relative_path: String = full_path
			.chars()
			.skip(crawl_path_len)
			.collect();

		paths.push(relative_path);

		//println!("{:?}", paths);
	}
	paths
}