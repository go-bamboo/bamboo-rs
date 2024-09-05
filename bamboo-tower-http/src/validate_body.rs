use std::fmt;
use std::marker::PhantomData;
use http::{header, Request, Response, StatusCode};
use http_body::Body;
use tower_http::validate_request::{ValidateRequest, ValidateRequestHeaderLayer};
use validator::Validate;

pub struct AcceptBody<ResBody> {
    _ty: PhantomData<fn() -> ResBody>,
}

impl<ResBody> AcceptBody<ResBody> {
    /// Create a new `AcceptBody`.
    ///
    /// # Panics
    ///
    /// Panics if `header_value` is not in the form: `type/subtype`, such as `application/json`
    pub fn new() -> Self
        where
            ResBody: Body + Default,
    {
        Self {
            _ty: PhantomData,
        }
    }
}

impl<ResBody> Clone for AcceptBody<ResBody> {
    fn clone(&self) -> Self {
        Self {
            _ty: PhantomData,
        }
    }
}

impl<ResBody> fmt::Debug for AcceptBody<ResBody> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AcceptHeader")
            // .field("header_value", &self.header_value)
            .finish()
    }
}

impl<B, ResBody> ValidateRequest<B> for AcceptBody<ResBody>
    where
        ResBody: Body + Default,
        B: Validate,
{
    type ResponseBody = ResBody;

    fn validate(&mut self, req: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        let b = req.body();
        if let Err(err) = b.validate() {
            let mut res = Response::new(ResBody::default());
            *res.status_mut() = StatusCode::NOT_ACCEPTABLE;
            return Err(res)
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use http::header;
    use tower::{BoxError, ServiceBuilder, ServiceExt};

    // #[tokio::test]
    // async fn valid_accept_header() {
    //     let mut service = ServiceBuilder::new()
    //         .layer(ValidateRequestHeaderLayer::accept("application/json"))
    //         .service_fn(echo);
    //
    //     let request = Request::get("/")
    //         .header(header::ACCEPT, "application/json")
    //         .body(Body::empty())
    //         .unwrap();
    //
    //     let res = service.ready().await.unwrap().call(request).await.unwrap();
    //
    //     assert_eq!(res.status(), StatusCode::OK);
    // }
}