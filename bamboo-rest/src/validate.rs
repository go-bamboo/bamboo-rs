use async_trait::async_trait;
use axum::extract::{FromRequest, Json, Request};
use axum::extract::rejection::JsonRejection;
use serde::de::DeserializeOwned;
use validator::Validate;

use bamboo_status::errors::Status;

// #[derive(Debug, Clone, Copy, Default)]
// pub struct ValidatedForm<T>(pub T);
//
// #[async_trait]
// impl<T, S> FromRequest<S> for ValidatedForm<T>
//     where
//         T: DeserializeOwned + Validate,
//         S: Send + Sync,
//         Form<T>: FromRequest<S, Rejection = FormRejection>,
// {
//     type Rejection = Status;
//
//     async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
//         let Form(value) = Form::<T>::from_request(req, state).await?;
//         value.validate()?;
//         Ok(ValidatedForm(value))
//     }
// }

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
    where
        T: DeserializeOwned + Validate,
        S: Send + Sync,
        Json<T>: FromRequest<S, Rejection=JsonRejection>,
{
    type Rejection = Status;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}