use std::convert::Infallible;

use askama_warp::Template;
use thiserror::Error;
use warp::{
    hyper::StatusCode,
    reject::{MethodNotAllowed, Reject},
    Rejection, Reply,
};

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("couldn't find entity")]
    NotFound,
    #[error("entity already exists")]
    AlreadyExists,
    #[error("rating {0} not within [0;5]")]
    RatingNotInRange(f32),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl Reject for ServiceError {}

impl From<base64::DecodeError> for ServiceError {
    fn from(e: base64::DecodeError) -> Self {
        ServiceError::Other(e.into())
    }
}

struct ErrMsg {
    statuscode: StatusCode,
    message: String,
}

impl ErrMsg {
    pub fn new(code: StatusCode, msg: &str) -> Self {
        ErrMsg {
            statuscode: code,
            message: msg.into(),
        }
    }

    pub fn into_reply(self) -> impl Reply {
        #[derive(Template)]
        #[template(path = "error.html")]
        struct ErrorTemplate {
            code: StatusCode,
            msg: String,
        }

        warp::reply::with_status(
            ErrorTemplate {
                code: self.statuscode,
                msg: self.message,
            },
            self.statuscode,
        )
    }
}

impl From<Rejection> for ErrMsg {
    fn from(r: Rejection) -> Self {
        if r.is_not_found() {
            return ErrMsg::new(StatusCode::NOT_FOUND, "Not found");
        }

        if let Some(service_error) = r.find::<ServiceError>() {
            return ErrMsg::from(service_error);
        }

        if r.find::<MethodNotAllowed>().is_some() {
            return ErrMsg::from(StatusCode::METHOD_NOT_ALLOWED);
        }

        ErrMsg::new(StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION")
    }
}

impl From<&ServiceError> for ErrMsg {
    fn from(e: &ServiceError) -> Self {
        match e {
            ServiceError::Unauthorized => ErrMsg::new(StatusCode::UNAUTHORIZED, "Unauthorized "),
            ServiceError::AlreadyExists => ErrMsg::new(StatusCode::CONFLICT, "Already exists"),
            ServiceError::NotFound => ErrMsg::new(StatusCode::NOT_FOUND, "Entity not found"),
            ServiceError::RatingNotInRange(r) => ErrMsg::new(
                StatusCode::BAD_REQUEST,
                &format!("Rating '{}' is not in [0;5]", r),
            ),
            _ => ErrMsg::new(StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION"),
        }
    }
}

impl From<StatusCode> for ErrMsg {
    fn from(sc: StatusCode) -> Self {
        match sc {
            StatusCode::UNAUTHORIZED => ErrMsg::new(sc, "Unauthorized"),
            StatusCode::METHOD_NOT_ALLOWED => ErrMsg::new(sc, "Method not Allowed"),
            _ => ErrMsg::new(StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION"),
        }
    }
}

pub async fn handle_rejection(r: Rejection) -> Result<impl Reply, Infallible> {
    Ok(ErrMsg::from(r).into_reply())
}
