use axum::{extract::FromRequestParts, http::request::Parts, response::IntoResponse};
use serde::Serialize;
use std::fmt::Debug;

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
    AuthorizationError,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        todo!()
    }
}
