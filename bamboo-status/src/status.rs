use std::fmt::{Display, Formatter};

use axum::{
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use serde_json::json;
use tonic::{
    Code,
    metadata::MetadataMap,
    transport::Error,
};
use tokio::sync::TryLockError;
use validator::{ValidationError, ValidationErrors};
use axum::extract::rejection::{FormRejection, JsonRejection};

use crate::errors::Status;
use crate::spring::SpringResponse;

pub type Result<T, E = Status> = std::result::Result<T, E>;

pub use anyhow::Result as AnyResult;


impl Status {
    pub fn new(reason: &str, message: &str) -> Status {
        Status {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
            reason: reason.to_string(),
            message: message.to_string(),
            metadata: Default::default(),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "code: {}, reason: {}, message: {} md: {:?}", self.code, self.reason, self.message, self.metadata)
    }
}

impl From<tonic::Status> for Status {
    fn from(value: tonic::Status) -> Self {
        let code = match value.code() {
            Code::Ok => StatusCode::OK,
            Code::Cancelled => StatusCode::NOT_FOUND,
            Code::Unknown => StatusCode::NOT_FOUND,
            Code::InvalidArgument => StatusCode::INTERNAL_SERVER_ERROR,
            Code::DeadlineExceeded => StatusCode::INTERNAL_SERVER_ERROR,
            Code::NotFound => StatusCode::INTERNAL_SERVER_ERROR,
            Code::AlreadyExists => StatusCode::INTERNAL_SERVER_ERROR,
            Code::PermissionDenied => StatusCode::INTERNAL_SERVER_ERROR,
            Code::ResourceExhausted => StatusCode::INTERNAL_SERVER_ERROR,
            Code::FailedPrecondition => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Aborted => StatusCode::INTERNAL_SERVER_ERROR,
            Code::OutOfRange => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unimplemented => StatusCode::NOT_FOUND,
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        };
        let mut s = Status {
            code: code.as_u16() as i32,
            reason: code.to_string(),
            message: value.message().to_string(),
            metadata: Default::default(),
        };
        if let Ok(ss) = serde_json::from_slice::<Status>(value.details()) {
            s.reason = ss.reason
        }
        s
    }
}

// impl From<TryLockError> for Status {
//     fn from(value: TryLockError) -> Self {
//         Status {
//             code: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
//             reason: crate::tran::ecode::ErrorReason::TryLockError.as_str_name().to_string(),
//             message: value.to_string(),
//             metadata: Default::default(),
//         }
//     }
// }
//
// impl From<tonic::transport::Error> for Status {
//     fn from(value: Error) -> Self {
//         Status {
//             code: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
//             reason: crate::tran::ecode::ErrorReason::TonicTransportErr.as_str_name().to_string(),
//             message: value.to_string(),
//             metadata: Default::default(),
//         }
//     }
// }
//
impl From<ValidationError> for Status {
    fn from(value: ValidationError) -> Self {
        Status {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
            reason: "ValidationError".to_string(),
            message: value.to_string(),
            metadata: Default::default(),
        }
    }
}

impl From<ValidationErrors> for Status {
    fn from(value: ValidationErrors) -> Self {
        Status {
            code: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
            reason: "ValidationErrors".to_string(),
            message: value.to_string(),
            metadata: Default::default(),
        }
    }
}
//
// impl From<FormRejection> for Status {
//     fn from(value: FormRejection) -> Self {
//         Status {
//             code: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
//             reason: crate::tran::ecode::ErrorReason::ValidationError.as_str_name().to_string(),
//             message: value.to_string(),
//             metadata: Default::default(),
//         }
//     }
// }
//
// impl From<JsonRejection> for Status {
//     fn from(value: JsonRejection) -> Self {
//         Status {
//             code: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as i32,
//             reason: crate::tran::ecode::ErrorReason::ValidationError.as_str_name().to_string(),
//             message: value.to_string(),
//             metadata: Default::default(),
//         }
//     }
// }

impl Into<tonic::Status> for Status {
    fn into(self) -> tonic::Status {
        let body = serde_json::to_vec(&self).unwrap();
        let mut mm = MetadataMap::new();
        // for (k, v) in self.metadata {
        //     mm.insert(k.parse().unwrap(), v.parse().unwrap());
        // }
        tonic::Status::with_details_and_metadata(Code::Internal, self.message, Bytes::from(body), mm)
    }
}

impl IntoResponse for Status {
    fn into_response(self) -> Response {
        let res = SpringResponse::new(false, self.reason, self.message, "");
        let body = Json(json!(res));
        (StatusCode::OK, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_eq() {
        let a = Status::new("UserNotFound", "");
        let b = Status::new("UserNotFound", "");
        assert_eq!(a, b);
    }

    #[test]
    fn test_status_eq1() {
        let a = Status::new("UserNotFound", "a");
        let b = Status::new("UserNotFound", "b");
        assert_ne!(a, b);
    }
}
