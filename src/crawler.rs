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

#[derive(Serialize, Deserialize)]
struct SourcePath {
    id: u32,
    full_path: String,
    status: String,
    num_photos: u32,
    size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct GalleryImage {
    source_path: String,
    relative_path: String,
    size: u64,
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
/// Extracts images from source
/// Goes recursively over all files in specified path and adds found jpegs to database
fn crawl_source(crawl_path: String, source_id: &u64) -> Result<bool, &'static str> {
    let source_path = crawl_path.clone();
    let relative_paths = get_paths_of_images(crawl_path);

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

    save_images_to_db(images, source_id)
}

/// Adds images to database
///
/// It saves only meta information about images to database
fn save_images_to_db(images: Vec<GalleryImage>, source_id: &u64) -> Result<bool, &'static str> {
    let connection = db::get_connection();

    for image in images.iter() {
        let result = connection.prep_exec(
            r"
		     INSERT INTO `photos` 
		             (`relative_path`, `source`, `filesize`) 
		     VALUES  (:relative_path, :source, :filesize)",
            params! {
                "relative_path" => image.relative_path.clone(),
                "source" => source_id,
                "filesize" => image.size
            },
        );

        match result {
            Ok(_) => (),
            Err(err) => println!("{:?}", err),
        }
    }
    Ok(true)
}

/// Extacts relative paths of images in specified directory recursively.
fn get_paths_of_images(search_path: String) -> Vec<String> {
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

pub fn get_photos(source_id: u64) -> HashMap<u64, String> {
    let connection = db::get_connection();

    // Select all photos from this source_id
    let result = connection
        .prep_exec(
            r"
		SELECT photos.id as id, CONCAT(`full_path`,`relative_path`) as 
		`full_path` FROM `photos`, `sources`
		WHERE sources.id=photos.source AND
		sources.id=:source_id",
            params! {"source_id" => source_id},
        )
        .unwrap();

    // We'll store images as pair id - absolute path
    let mut images: HashMap<u64, String> = HashMap::new();

    // Convert query resuts to HashMap
    result.for_each(|row| match row {
        Ok(row) => {
            let (id, full_path): (u64, String) = my::from_row(row);
            images.insert(id, full_path);
        }
        Err(_) => {}
    });
    println!("images list size: {:?}", images.len());
    return images;
}

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
