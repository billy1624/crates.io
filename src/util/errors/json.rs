use axum::response::{IntoResponse, Response};
use axum::Json;
use std::borrow::Cow;
use std::fmt;

use super::{AppError, BoxedAppError, InternalAppErrorStatic};

use crate::rate_limiter::LimitedAction;
use chrono::NaiveDateTime;
use http::{header, StatusCode};

/// Generates a response with the provided status and description as JSON
fn json_error(detail: &str, status: StatusCode) -> Response {
    let json = json!({ "errors": [{ "detail": detail }] });
    (status, Json(json)).into_response()
}

// The following structs are empty and do not provide a custom message to the user

#[derive(Debug)]
pub(crate) struct ReadOnlyMode;

impl AppError for ReadOnlyMode {
    fn response(&self) -> Response {
        let detail = "crates.io is currently in read-only mode for maintenance. \
                      Please try again later.";
        json_error(detail, StatusCode::SERVICE_UNAVAILABLE)
    }
}

impl fmt::Display for ReadOnlyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Tried to write in read only mode".fmt(f)
    }
}

// The following structs wrap owned data and provide a custom message to the user

pub fn custom(status: StatusCode, detail: impl Into<Cow<'static, str>>) -> BoxedAppError {
    Box::new(CustomApiError {
        status,
        detail: detail.into(),
    })
}

#[derive(Debug, Clone)]
pub struct CustomApiError {
    status: StatusCode,
    detail: Cow<'static, str>,
}

impl fmt::Display for CustomApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.detail.fmt(f)
    }
}

impl AppError for CustomApiError {
    fn response(&self) -> Response {
        json_error(&self.detail, self.status)
    }
}

#[derive(Debug)]
pub(crate) struct TooManyRequests {
    pub action: LimitedAction,
    pub retry_after: NaiveDateTime,
}

impl AppError for TooManyRequests {
    fn response(&self) -> Response {
        const HTTP_DATE_FORMAT: &str = "%a, %d %b %Y %H:%M:%S GMT";
        let retry_after = self.retry_after.format(HTTP_DATE_FORMAT);

        let detail = format!(
            "{}. Please try again after {retry_after} or email \
             help@crates.io to have your limit increased.",
            self.action.error_message()
        );
        let mut response = json_error(&detail, StatusCode::TOO_MANY_REQUESTS);
        response.headers_mut().insert(
            header::RETRY_AFTER,
            retry_after
                .to_string()
                .try_into()
                .expect("HTTP_DATE_FORMAT contains invalid char"),
        );
        response
    }
}

impl fmt::Display for TooManyRequests {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "Too many requests".fmt(f)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InsecurelyGeneratedTokenRevoked;

impl InsecurelyGeneratedTokenRevoked {
    pub fn boxed() -> BoxedAppError {
        Box::new(Self)
    }
}

impl AppError for InsecurelyGeneratedTokenRevoked {
    fn response(&self) -> Response {
        json_error(&self.to_string(), StatusCode::UNAUTHORIZED)
    }

    fn cause(&self) -> Option<&dyn AppError> {
        Some(&InternalAppErrorStatic {
            description: "insecurely generated, revoked 2020-07",
        })
    }
}

pub const TOKEN_FORMAT_ERROR: &str =
    "The given API token does not match the format used by crates.io. \
    \
    Tokens generated before 2020-07-14 were generated with an insecure \
    random number generator, and have been revoked. You can generate a \
    new token at https://crates.io/me. \
    \
    For more information please see \
    https://blog.rust-lang.org/2020/07/14/crates-io-security-advisory.html. \
    We apologize for any inconvenience.";

impl fmt::Display for InsecurelyGeneratedTokenRevoked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(TOKEN_FORMAT_ERROR)?;
        Result::Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct OwnershipInvitationExpired {
    pub(crate) crate_name: String,
}

impl AppError for OwnershipInvitationExpired {
    fn response(&self) -> Response {
        json_error(&self.to_string(), StatusCode::GONE)
    }
}

impl fmt::Display for OwnershipInvitationExpired {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The invitation to become an owner of the {} crate expired. \
             Please reach out to an owner of the crate to request a new invitation.",
            self.crate_name
        )
    }
}

#[derive(Debug)]
pub(crate) struct MetricsDisabled;

impl AppError for MetricsDisabled {
    fn response(&self) -> Response {
        json_error(&self.to_string(), StatusCode::NOT_FOUND)
    }
}

impl fmt::Display for MetricsDisabled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Metrics are disabled on this crates.io instance")
    }
}
