//! Integrate twoface with other libraries, like Actix-web or Diesel.

use crate::twoface::TfError;
use actix_web::{
    http::{header, StatusCode},
    HttpResponse,
};
use serde::Serialize;
use tracing::error;

// Twoface errors can be used as Actix-web errors.
// If a handler returns a Twoface error, the external portion will be shown to the user.
// The internal portion will only be logged.
impl actix_web::ResponseError for TfError {
    fn status_code(&self) -> StatusCode {
        self.external.cause.into()
    }

    fn error_response(&self) -> HttpResponse {
        error!("{}", self.internal);
        let resp = serde_json::to_string(&ErrBody {
            error: self.to_string(),
        })
        .unwrap_or_else(|e| {
            error!("Serde error: {}", e.to_string());
            "{\"error\": \"ServerError: internal server error\"}".to_owned()
        });
        HttpResponse::build(self.external.cause.into())
            .header(header::CONTENT_TYPE, "application/json")
            .body(resp)
    }
}

#[derive(Serialize)]
struct ErrBody {
    error: String,
}

#[cfg(test)]
mod tests {
    use crate::twoface::externalerror::Cause;
    use crate::twoface::*;
    use actix_web::{dev::Service, test, web, App, Error as ActixError};

    #[actix_rt::test]
    async fn test() -> Result<(), ActixError> {
        async fn index() -> Fallible<web::Json<String>> {
            let file = std::fs::read_to_string("secret-filename-do-not-leak-to-user");
            file.describe_err(ExternalError {
                cause: Cause::ServerError,
                text: "page not found",
            })
            .map(web::Json)
        }

        let mut app = test::init_service(App::new().service(web::resource("/").route(web::get().to(index)))).await;

        // Send a request
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = app.call(req).await.unwrap();

        let expected_body = "{\"error\":\"ServerError: page not found\"}";
        if let Some(actix_web::body::Body::Bytes(bytes)) = resp.response().body().as_ref() {
            let actual_body = String::from_utf8(bytes.to_vec()).unwrap();
            assert_eq!(actual_body, expected_body);
        } else {
            panic!("wrong response type");
        }
        Ok(())
    }
}
