use std::net::TcpListener;

use serde_json::json;
use sqlx::{Connection, PgConnection};

use space_telescope::configuration::get_configuration;

/// Spin up instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TcpListener.");
    let port = listener.local_addr().unwrap().port();
    let server = space_telescope::startup::run(listener).expect("Failed to bind address");

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn test_health_check_success() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn test_post_renders_returns_202_for_valid_body_fields() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let db_connection_string = configuration.database.connection_string();
    let mut db_connection = PgConnection::connect(&db_connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    // Act
    let body = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane_bases": [
            [1f32, 0f32, 0f32],
            [0f32, 1f32, 0f32],
        ],
        "primary_direction": [1f32, 0f32, 0f32],
        "observer_position": [0f32, 0f32, 0f32],
        "latitude": -45f32,
        "longitude": 120f32,
        "filters": [
            "SDSS_U",
            "SDSS_G",
            "SDSS_R",
            0.55555f32,
        ],
    });
    let response = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(202, response.status().as_u16());

    let render = sqlx::query!("SELECT * FROM renders")
        .fetch_one(&mut db_connection)
        .await
        .expect("Failed to fetch queued render.");

    assert_eq!(render.image_url, None);
    assert_eq!(render.fov_x, body["fov"][0]);
    assert_eq!(render.fov_y, body["fov"][1]);
    assert_eq!(render.image_dimension_x, body["image_dimensions"][0]);
    assert_eq!(render.image_dimension_y, body["image_dimensions"][1]);
    assert_eq!(
        render.fundamental_plane_bases,
        vec![1f32, 0f32, 0f32, 0f32, 1f32, 0f32,]
    );
    assert_eq!(render.primary_direction, vec![1f32, 0f32, 0f32]);
    assert_eq!(render.observer_position, vec![0f32, 0f32, 0f32]);
    assert_eq!(render.latitude, body["latitude"]);
    assert_eq!(render.longitude, body["longitude"]);
    assert_eq!(
        render.broadband_filters,
        Some(vec![
            "SDSS_U".to_string(),
            "SDSS_G".to_string(),
            "SDSS_R".to_string()
        ])
    );
    assert_eq!(render.narrowband_filters, Some(vec![0.55555f32]));
}

#[tokio::test]
async fn test_post_renders_returns_400_for_missing_body_fields() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane_bases": [
            [1f32, 0f32, 0f32],
            [0f32, 1f32, 0f32],
        ],
        "primary_direction": [1f32, 0f32, 0f32],
        "observer_position": [0f32, 0f32, 0f32],
        "latitude": -45f32,
        // Oops! Forgot the longitude
        "filters": [
            "SDSS_U",
            "SDSS_G",
            "SDSS_R",
            0.55555f32,
        ],
    });
    let response = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn test_post_renders_returns_400_for_nonpositive_fov_values() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body_zero = json!({
        "fov": [50f32, 0f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane_bases": [
            [1f32, 0f32, 0f32],
            [0f32, 1f32, 0f32],
        ],
        "primary_direction": [1f32, 0f32, 0f32],
        "observer_position": [0f32, 0f32, 0f32],
        "latitude": -45f32,
        "longitude": 120f32,
        "filters": [
            "SDSS_U",
            "SDSS_G",
            "SDSS_R",
            0.55555f32,
        ],
    });
    let body_negative = json!({
        "fov": [-50f32, 50f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane_bases": [
            [1f32, 0f32, 0f32],
            [0f32, 1f32, 0f32],
        ],
        "primary_direction": [1f32, 0f32, 0f32],
        "observer_position": [0f32, 0f32, 0f32],
        "latitude": -45f32,
        "longitude": 120f32,
        "filters": [
            "SDSS_U",
            "SDSS_G",
            "SDSS_R",
            0.55555f32,
        ],
    });
    let response_zero = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_zero.to_string())
        .send()
        .await
        .expect("Failed to execute request.");
    let response_negative = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_negative.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(400, response_zero.status().as_u16());
    assert_eq!(400, response_negative.status().as_u16());
}

#[tokio::test]
async fn test_post_renders_returns_400_for_parallel_fundamental_plane_vectors() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane_bases": [
            [0.23456f32, 0f32, 0.3f32],
            [0.46912f32, 0f32, 0.6f32],
        ],
        "primary_direction": [1f32, 0f32, 0f32],
        "observer_position": [0f32, 0f32, 0f32],
        "latitude": -45f32,
        "longitude": 120f32,
        "filters": [
            "SDSS_U",
            "SDSS_G",
            "SDSS_R",
            0.55555f32,
        ],
    });
    let response = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn test_post_renders_returns_400_for_latitude_out_of_range() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body_negative = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane_bases": [
            [1f32, 0f32, 0f32],
            [0f32, 1f32, 0f32],
        ],
        "primary_direction": [1f32, 0f32, 0f32],
        "observer_position": [0f32, 0f32, 0f32],
        "latitude": -91f32,
        "longitude": 120f32,
        "filters": [
            "SDSS_U",
            "SDSS_G",
            "SDSS_R",
            0.55555f32,
        ],
    });
    let body_positive = json!({
        "fov": [50f32, 50f32],
        "image_dimensions": [256u32, 256u32],
        "fundamental_plane_bases": [
            [1f32, 0f32, 0f32],
            [0f32, 1f32, 0f32],
        ],
        "primary_direction": [1f32, 0f32, 0f32],
        "observer_position": [0f32, 0f32, 0f32],
        "latitude": 91f32,
        "longitude": 120f32,
        "filters": [
            "SDSS_U",
            "SDSS_G",
            "SDSS_R",
            0.55555f32,
        ],
    });
    let response_negative = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_negative.to_string())
        .send()
        .await
        .expect("Failed to execute request.");
    let response_positive = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_positive.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(400, response_negative.status().as_u16());
    assert_eq!(400, response_positive.status().as_u16());
}
