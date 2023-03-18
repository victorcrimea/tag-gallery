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
    openapi_get_routes_spec![settings: add_source_path, list_source_paths, list_photos]
}

/// Adds source path to the database.
///
/// This function saves provided absolute path (on the server) to the database
/// and goes over all jpeg files recursively in order to add them to DB.

#[openapi(tag = "Crawler")]
#[post("/add_source_path?<path>")]
pub fn add_source_path(mut db: Connection<MainDB>, path: &str) -> crate::ApiResult<()> {
    // let params = request.get_ref::<Params>().unwrap();

    // let path = &params["path"];

    // let connection = db::get_connection();
    // let result = connection.prep_exec(
    //     r"
    //   INSERT INTO `sources`
    //           (`full_path`)
    //   VALUES  (:path)",
    //     params! {"path" => String::from_value(path)},
    // );

    // let mut source_id: u64 = 0;
    // match result {
    //     Ok(result) => {
    //         source_id = result.last_insert_id();
    //     }
    //     Err(_) => (),
    // }

    // match crawl_source(String::from_value(path).unwrap(), &source_id) {
    //     Ok(_) => {
    //         // Source was successfully crawled
    //         let _result = connection.prep_exec(
    //             r"
    //     UPDATE `sources`
    //     SET   `status` = 'indexed'
    //     WHERE `id` = :source_id",
    //             params! {"source_id" => &source_id},
    //         );
    //         Ok(Response::with((status::Ok, "ok")))
    //     }
    //     Err(err) => Ok(Response::with((
    //         status::Ok,
    //         "Error: cannot crawl: {:?}",
    //         err,
    //     ))),
    // }

    todo!()
}

/// Provides all available source paths
#[openapi(tag = "Crawler")]
#[get("/list_source_paths")]
pub fn list_source_paths(mut db: Connection<MainDB>) -> crate::ApiResult<()> {
    //   let connection = db::get_connection();
    //   let result = connection
    //       .prep_exec(
    //           r"
    // SELECT sources.id,
    //        full_path,
    //        status,
    //        count(photos.id) as num_photos,
    //        sum(filesize) as size
    // FROM `sources`
    // LEFT JOIN `photos` on photos.source = sources.id
    // GROUP BY sources.id",
    //           (),
    //       )
    //       .unwrap();

    //   let mut paths: Vec<SourcePath> = vec![];

    //   result.for_each(|row| match row {
    //       Ok(row) => {
    //           let (id, full_path, status, num_photos, size) = my::from_row(row);
    //           paths.push(SourcePath {
    //               id: id,
    //               full_path: full_path,
    //               status: status,
    //               num_photos: num_photos,
    //               size: size,
    //           });
    //       }
    //       Err(_) => {}
    //   });

    //   let out_json = json!({
    //       "paths": paths,
    //   });

    //   Ok(Response::with((
    //       status::Ok,
    //       to_string_pretty(&out_json).unwrap(),
    //   )))
    todo!()
}

#[openapi(tag = "Crawler")]
#[get("/list_photos/<id>")]
pub fn list_photos(mut db: Connection<MainDB>, id: &str) -> crate::ApiResult<()> {
    // let ref id = request
    //     .extensions
    //     .get::<Router>()
    //     .unwrap()
    //     .find("id")
    //     .unwrap_or("0");

    // let source_id = id.parse::<u64>().unwrap_or(0);

    // let mut ids: Vec<u64> = vec![];

    // for key in get_photos(source_id).keys() {
    //     ids.push(*key);
    // }

    // let out_json = json!({
    //     "photos": ids,
    // });

    // Ok(Response::with((
    //     status::Ok,
    //     to_string_pretty(&out_json).unwrap(),
    // )))
    todo!()
}
