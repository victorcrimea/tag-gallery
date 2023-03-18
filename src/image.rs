// Stndard includes

use persistent::State;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

// Library includes
use iron::prelude::*;
use iron::status;
use mysql as my;
use mysql::chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use router::Router;

use serde_json::to_string_pretty;

// Local includes
use crawler;
use db;
use Settings;

pub fn info(request: &mut Request) -> IronResult<Response> {
    // Get url params
    let ref id = request
        .extensions
        .get::<Router>()
        .unwrap()
        .find("id")
        .unwrap_or("0");

    let photo_id: u64 = id.parse::<u64>().unwrap_or(0);

    // Check if photo exists
    let connection = db::get_connection();
    let result = connection.prep_exec(
        r"
	    SELECT id,
	           filesize,
	           exif_gps_date,
	           exif_gps_time,
	           UNIX_TIMESTAMP(exif_datetime) as `exif_unix_timestamp`,
	           exif_latitude,
	           exif_longitude,
	           exif_altitude
	    FROM `photos`
	    WHERE photos.id = :id",
        params! {"id" => photo_id},
    );

    match result {
        Ok(result) => {
            // We get here only if image exists in DB
            let mut out_json = json!({});
            result.for_each(|row| {
                match row {
                    Ok(mut row) => {
                        let size: u64 = row.take("size").unwrap_or(0);
                        let gps_date: NaiveDate = row
                            .take("exif_gps_date")
                            .unwrap_or(NaiveDate::from_ymd(1970, 1, 1));
                        let gps_date: String = gps_date.to_string();
                        let gps_time: NaiveTime = row
                            .take("exif_gps_time")
                            .unwrap_or(NaiveTime::from_hms(0, 0, 0));
                        let gps_time: String = gps_time.to_string();
                        let exif_timestamp: i64 = row.take("exif_unix_timestamp").unwrap_or(0);
                        //let exif_timestamp: String = exif_timestamp.to_string();
                        let latitude: f64 = row.take("exif_latitude").unwrap_or(0.0);
                        let longitude: f64 = row.take("exif_longitude").unwrap_or(0.0);
                        let altitude: f64 = row.take("exif_altitude").unwrap_or(0.0);
                        out_json = json!({
                            "size" : size,
                            "gps_date" : gps_date,
                            "gps_time" : gps_time,
                            "exif_timestamp" : exif_timestamp,
                            "latitude": latitude,
                            "longitude": longitude,
                            "altitude": altitude
                        });
                    }
                    Err(_) => {}
                }
            });

            Ok(Response::with((
                status::Ok,
                to_string_pretty(&out_json).unwrap(),
            )))
        }
        Err(_) => Ok(Response::with((status::NotFound, ""))),
    }
}

fn read_image(gallery_folder: &str, id: u64, size: &str) -> Option<Vec<u8>> {
    let mut buffer: Vec<u8> = vec![];
    let file = File::open(format!("{}/{}/{}.jpg", gallery_folder, size, id));

    match file {
        Ok(mut file) => match file.read_to_end(&mut buffer) {
            Ok(_) => {
                return Some(buffer);
            }
            Err(_) => {
                return None;
            }
        },
        Err(_) => return None,
    }
}
