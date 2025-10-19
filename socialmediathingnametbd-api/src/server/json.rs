use crate::server::ServerError;
use axum::{
    Json as AxumJson,
    extract::FromRequest,
    response::{IntoResponse, Response},
};
use axum_extra::TypedHeader;
use headers::ContentType;
use serde::Serialize;

#[derive(FromRequest, Debug, Clone, Copy, Default)]
#[from_request(via(AxumJson), rejection(ServerError))]
pub struct Json<T>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(json) => (TypedHeader(ContentType::json()), json).into_response(),
            Err(err) => ServerError::JsonResponse(err).into_response(),
        }
    }
}
