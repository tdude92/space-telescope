use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use nalgebra as na;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// TODO fill this out
#[derive(serde::Deserialize)]
#[allow(non_camel_case_types)]
enum BroadBandFilter {
    SDSS_U,
    SDSS_G,
    SDSS_R,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum AstronomicalFilter {
    NarrowBand(f32),
    BroadBand(BroadBandFilter),
}

// TODO impl utoipa::ToSchema
#[derive(serde::Deserialize)]
struct RenderJob {
    fov: [f32; 2],
    image_dimensions: [u32; 2],
    fundamental_plane_bases: [na::Vector3<f32>; 2],
    primary_direction: na::Vector3<f32>,
    observer_position: na::Vector3<f32>,
    latitude: f32,
    longitude: f32,
    filters: Vec<AstronomicalFilter>,
}

#[utoipa::path(
    get,
    path = "/health_check",
    responses((status = 200, description = "Service is running as expected.")),
)]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[utoipa::path(
    post,
    path = "/renders",
    request_body = RenderJob,
    responses(
        (status = 202, description = "Render job successfully queued."),
        (status = 400, description = "Render job request body malformed.")
    )
)]
async fn submit_render_request(body: web::Json<RenderJob>) -> impl Responder {
    // TODO modulo longitude 360
    // TODO project primary direction onto fundamental plane
    if body.latitude > 90.0 || body.latitude < -90.0 {
        HttpResponse::BadRequest()
    } else if body.fov[0] <= 0.0 || body.fov[1] <= 0.0 {
        HttpResponse::BadRequest()
    } else if body.fundamental_plane_bases[0]
        .cross(&body.fundamental_plane_bases[1])
        .norm_squared()
        == 0.0
    {
        HttpResponse::BadRequest()
    } else {
        HttpResponse::Accepted()
    }
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    #[derive(OpenApi)]
    #[openapi(
        info(description = "space_telescope backend API."),
        paths(health_check, submit_render_request)
    )]
    struct ApiDoc;

    let server = HttpServer::new(move || {
        App::new()
            .service(SwaggerUi::new("/docs/{_:.*}").url("/openapi.json", ApiDoc::openapi()))
            .route("/health_check", web::get().to(health_check))
            .route("/renders", web::post().to(submit_render_request))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
