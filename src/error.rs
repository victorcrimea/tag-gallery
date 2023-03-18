use std::fmt;
use std::fmt::Formatter;

use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::okapi::schemars::Map;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::OpenApiError;
use serde_repr::*;

pub mod conversion;

#[derive(Debug, Serialize_repr)]
#[repr(u32)]
pub enum ApiErrorCode {
    /// Low-level errors
    IoError = 100,
    ParseError = 200,

    /// Main errors
    DatabaseError = 1001,
    ValueError = 1005,
    UserNotExists = 1008,
}

/// Error messages returned to user
#[derive(Debug, serde::Serialize)]
pub struct ApiError {
    /// The title of the error message
    pub error: String,
    /// The description of the error
    pub message: Option<String>,
    /// Adozi-specific error code
    pub error_code: ApiErrorCode,
    // HTTP Status Code returned, used by Rocket, not serialized.
    #[serde(skip)]
    pub http_status_code: u16,
}

impl OpenApiResponderInner for ApiError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "400".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
				# [400 Bad Request](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/400)\n\
				The request given is wrongly formatted or data asked could not be fulfilled. \
				"
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
				# [404 Not Found](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/404)\n\
				This response is given when you request a page that does not exists.\
				"
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
				# [422 Unprocessable Entity](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/422)\n\
				This response is given when you request body is not correctly formatted. \
				"
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "450".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
				# [450 Adozi Api Error]\n\
				This response is given when there is an error in request \
				"
                .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "500".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "\
				# [500 Internal Server Error](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/500)\n\
				This response is given when something wend wrong on the server. \
				"
                .to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Error `{}`: {}",
            self.error,
            self.message.as_deref().unwrap_or("<no message>")
        )
    }
}

impl fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
