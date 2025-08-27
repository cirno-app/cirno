use axum::{
    Json, RequestPartsExt,
    extract::{FromRequest, FromRequestParts},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use log::info;
use serde::Serialize;
use std::fmt::Debug;
use wry::http::StatusCode;

#[derive(FromRequest)]
#[from_request(via(Json), rejection(ApiError))]
struct ApiJson<T>(T);

impl<T> IntoResponse for ApiJson<T>
where
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

#[derive(Debug, Serialize)]
struct ResponseBody<D>
where
    D: Debug + Serialize,
{
    code: u16,
    data: D,
}

impl<D> ResponseBody<D>
where
    D: Debug + Serialize,
{
    fn new(data: D) -> ResponseBody<D> {
        ResponseBody { code: 200, data }
    }
}

struct AppClaim {}

impl<S> FromRequestParts<S> for AppClaim
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|err| {
                info!("");
                ApiError::AuthorizationError
            })?;
        // Return "403 Forbidden" for both authorization and authentication failures,
        // but also log the specific reason for the failure.
        // This prevents attackers from obtaining detailed information,
        // while allowing service owners to troubleshoot in logs.

        todo!()
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponseBody {
    code: u16,
    msg: String,
}

enum ApiError {
    InvalidJsonError,
    AuthorizationError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            ApiError::InvalidJsonError => (StatusCode::BAD_REQUEST.as_u16(), "".to_owned()),
            ApiError::AuthorizationError => (StatusCode::FORBIDDEN.as_u16(), "".to_owned()),
        };

        (StatusCode::OK, ApiJson(ErrorResponseBody { code, msg })).into_response()
    }
}
