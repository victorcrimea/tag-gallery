use std::io::Error;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response;
use rocket::response::Responder;
use rocket::response::Response;

use validator::ValidationErrors;

use super::ApiError;
use super::ApiErrorCode;

impl std::error::Error for ApiError {}

impl From<rocket::serde::json::Error<'_>> for ApiError {
    fn from(err: rocket::serde::json::Error) -> Self {
        use rocket::serde::json::Error::*;
        match err {
            Io(io_error) => ApiError {
                error: "IO Error".to_owned(),
                error_code: ApiErrorCode::IoError,
                message: Some(io_error.to_string()),
                http_status_code: 422,
            },
            Parse(_raw_data, parse_error) => ApiError {
                error: "Parse Error".to_owned(),
                error_code: ApiErrorCode::ParseError,
                message: Some(parse_error.to_string()),
                http_status_code: 422,
            },
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // Convert object to json
        let body = serde_json::to_string(&self).unwrap();
        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(Status::new(self.http_status_code))
            .ok()
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(err: ValidationErrors) -> Self {
        ApiError {
            error: "Parse Error".to_owned(),
            error_code: ApiErrorCode::ParseError,
            message: Some(err.to_string()),
            http_status_code: 422,
        }
    }
}

impl From<std::io::Error> for ApiError {
    fn from(_: Error) -> Self {
        ApiError {
            error: "IO Error".to_owned(),
            error_code: ApiErrorCode::IoError,
            message: Some("IO Error".to_string()),
            http_status_code: 450,
        }
    }
}
