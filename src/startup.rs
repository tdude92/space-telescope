use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::health_check::{__path_health_check, health_check};
use crate::routes::renders::{__path_submit_render_request, submit_render_request};

pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    #[derive(OpenApi)]
    #[openapi(
        info(description = "space-telescope backend API."),
        paths(health_check, submit_render_request)
    )]
    struct ApiDoc;

    let db_pool = Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .service(SwaggerUi::new("/docs/{_:.*}").url("/openapi.json", ApiDoc::openapi()))
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/renders", web::post().to(submit_render_request))
            .app_data(db_pool.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
