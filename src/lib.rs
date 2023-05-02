use std::net::TcpListener;

use actix_web::{
    web,
    App,
    HttpRequest,
    HttpResponse,
    HttpServer,
    Responder,
};
use actix_web::dev::Server;
use utoipa::{OpenApi};
use utoipa_swagger_ui::SwaggerUi;

#[utoipa::path(
    get,
    path = "/health_check",
    responses((status = 200, description = "Service is running as expected.")),
)]
async fn health_check(_: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    #[derive(OpenApi)]
    #[openapi(
        info(description = "space_telescope backend API."),
        paths(health_check),
    )]
    struct ApiDoc;

    let server = HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/docs/{_:.*}")
                    .url("/openapi.json", ApiDoc::openapi())
            )
            .route("/health_check", web::get().to(health_check))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
