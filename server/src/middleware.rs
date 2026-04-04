use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    http::Method,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;

use crate::service::auth::verify_token;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareInner<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareInner {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareInner<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareInner<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path();
        if path == "/login" || (path == "/user" && req.method() == Method::POST) {
            let fut = self.service.call(req);
            return Box::pin(fut);
        }

        let token = req
            .headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| header.strip_prefix("Bearer "));

        match token {
            Some(token) => match verify_token(token) {
                Ok(user_id) => {
                    req.extensions_mut().insert(user_id);
                    let fut = self.service.call(req);
                    Box::pin(fut)
                }
                Err(_) => {
                    Box::pin(async move { Err(ErrorUnauthorized("Invalid or expired token")) })
                }
            },
            None => Box::pin(async move { Err(ErrorUnauthorized("Missing Authorization header")) }),
        }
    }
}
