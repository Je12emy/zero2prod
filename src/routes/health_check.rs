use actix_web::{HttpRequest, Responder, HttpResponse};

pub async fn health_check(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}
