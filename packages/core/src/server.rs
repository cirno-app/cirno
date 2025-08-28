use crate::AppState;
use axum::{
    Json, RequestPartsExt,
    extract::{FromRequest, FromRequestParts, rejection::JsonRejection},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use log::info;
use serde::Serialize;
use std::{fmt::Debug, sync::Arc};
use wry::http::StatusCode;

pub mod controller;

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

pub struct AppClaim {}

impl FromRequestParts<Arc<AppState>> for AppClaim {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            // Return "403 Forbidden" for both authorization and authentication failures,
            // but also log the specific reason for the failure.
            // This prevents attackers from obtaining detailed information,
            // while allowing service owners to troubleshoot in logs.
            .map_err(|err| {
                info!("");
                ApiError::AuthorizationError
            })?;

        todo!()
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponseBody {
    code: u16,
    msg: String,
}

pub enum ApiError {
    InvalidJsonError(String),
    AuthorizationError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            ApiError::InvalidJsonError(body) => (StatusCode::BAD_REQUEST.as_u16(), body),
            ApiError::AuthorizationError => (StatusCode::FORBIDDEN.as_u16(), "".to_owned()),
        };

        (StatusCode::OK, ApiJson(ErrorResponseBody { code, msg })).into_response()
    }
}

impl From<JsonRejection> for ApiError {
    fn from(value: JsonRejection) -> Self {
        ApiError::InvalidJsonError(value.body_text())
    }
}
