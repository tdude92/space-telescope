use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use nalgebra as na;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

// TODO fill this out
#[derive(serde::Deserialize, Debug)]
#[allow(non_camel_case_types)]
enum BroadBandFilter {
    SDSS_U,
    SDSS_G,
    SDSS_R,
}

impl std::fmt::Display for BroadBandFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum AstronomicalFilter {
    NarrowBand(f32),
    BroadBand(BroadBandFilter),
}

#[derive(serde::Deserialize)]
struct FundamentalPlane {
    basis: [na::Vector3<f32>; 2],
}

impl FundamentalPlane {
    pub fn basis_vec_1(&self) -> Vec<f32> {
        self.basis[0].data.as_slice().to_vec()
    }

    pub fn basis_vec_2(&self) -> Vec<f32> {
        self.basis[1].data.as_slice().to_vec()
    }
}

// TODO impl utoipa::ToSchema
#[derive(serde::Deserialize)]
pub struct RenderJob {
    email: String,
    fov: [f32; 2],
    image_dimensions: [i32; 2],
    fundamental_plane: FundamentalPlane,
    observer_position: na::Vector3<f32>,
    latitude: f32,
    longitude: f32,
    filters: Vec<AstronomicalFilter>,
}

impl RenderJob {
    pub fn narrowband_filters(&self) -> Vec<f32> {
        let mut output = vec![];
        for filter in &self.filters {
            if let AstronomicalFilter::NarrowBand(wavelength) = filter {
                output.push(*wavelength);
            }
        }
        output
    }

    pub fn broadband_filters(&self) -> Vec<String> {
        let mut output = vec![];
        for filter in &self.filters {
            if let AstronomicalFilter::BroadBand(filter_name) = filter {
                output.push(filter_name.to_string());
            }
        }
        output
    }
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
pub async fn submit_render_request(
    body: web::Json<RenderJob>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    // TODO Input validation
    // -90 <= lat <= 90
    // fov_x, fov_y > 0
    // fundamental bases are not parallel vectors
    // modulo longitude 360
    // project primary direction onto fundamental plane
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Inserting new render job into queue",
        %request_id
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new render job details in the database");
    let render_id = Uuid::new_v4();
    match sqlx::query!(
        r#"
        INSERT INTO renders (
            id,
            created_at,
            email,
            fov_x,
            fov_y,
            image_dimension_x,
            image_dimension_y,
            fundamental_plane_basis_vector_1,
            fundamental_plane_basis_vector_2,
            observer_position,
            latitude,
            longitude,
            narrowband_filters,
            broadband_filters
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        render_id,
        Utc::now(),
        body.email,
        body.fov[0],
        body.fov[1],
        body.image_dimensions[0],
        body.image_dimensions[1],
        &body.fundamental_plane.basis_vec_1(),
        &body.fundamental_plane.basis_vec_2(),
        &body.observer_position.data.as_slice().to_vec(),
        body.latitude,
        body.longitude,
        &body.narrowband_filters(),
        &body.broadband_filters(),
    )
    .execute(db_pool.as_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Accepted().finish(),
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
