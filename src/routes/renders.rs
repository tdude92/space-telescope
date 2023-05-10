use actix_web::{web, HttpResponse, HttpResponseBuilder, Responder};
use nalgebra as na;
use serde_json::json;

/// Create [`HttpResponse`] with an error body.
///
/// `http_response` is generally an [`HttpResponseBuilder`] with an HTTP status code.
fn error_response(mut http_response: HttpResponseBuilder, error_message: &str) -> HttpResponse {
    http_response.json(json!({ "error_message": error_message }))
}

/// Check that -90 <= latitude <= 90
fn is_latitude_valid(latitude: f32) -> bool {
    latitude > 90.0 || latitude < -90.0
}

/// Check that fov values are positive
fn is_fov_valid(fov_x: f32, fov_y: f32) -> bool {
    fov_x <= 0.0 || fov_y <= 0.0
}

/// Check that fundamental plane bases are not parallel
fn is_fundamental_bases_valid(v1: &na::Vector3<f32>, v2: &na::Vector3<f32>) -> bool {
    v1.cross(v2).norm_squared() == 0.0
}

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
pub struct RenderJob {
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
    post,
    path = "/renders",
    request_body = RenderJob,
    responses(
        (status = 202, description = "Render job successfully queued."),
        (status = 400, description = "Render job request body malformed.")
    )
)]
pub async fn submit_render_request(body: web::Json<RenderJob>) -> impl Responder {
    // TODO modulo longitude 360
    // TODO project primary direction onto fundamental plane
    if is_latitude_valid(body.latitude) {
        error_response(
            HttpResponse::BadRequest(),
            &format!("Latitude is invalid with value: {} deg.", body.latitude),
        )
    } else if is_fov_valid(body.fov[0], body.fov[1]) {
        error_response(
            HttpResponse::BadRequest(),
            &format!("FOV is invalid with x={} y={}.", body.fov[0], body.fov[1]),
        )
    } else if is_fundamental_bases_valid(
        &body.fundamental_plane_bases[0],
        &body.fundamental_plane_bases[1],
    ) {
        error_response(
            HttpResponse::BadRequest(),
            "Fundamental plane bases are parallel.",
        )
    } else {
        HttpResponse::Accepted().finish()
    }
}
