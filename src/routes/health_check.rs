use actix_web::HttpResponse;

#[utoipa::path(
    get,
    path = "/health_check",
    responses((status = 200, description = "Service is running as expected.")),
)]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
