use std::net::TcpListener;

use serde_json::json;

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
    let body = json!({
        "fov": [50f32, 50f32],
        "image_dimensions": [256u32, 256u32],
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
    })
    .to_string();

    // Act
    let response = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(202, response.status().as_u16());
}

#[tokio::test]
async fn test_post_renders_returns_400_for_missing_body_fields() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let body = json!({
        "fov": [50f32, 50f32],
        "image_dimensions": [256u32, 256u32],
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
    })
    .to_string();

    // Act
    let response = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body)
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
    let body_zero = json!({
        "fov": [50f32, 0f32],
        "image_dimensions": [256u32, 256u32],
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
    })
    .to_string();
    let body_negative = json!({
        "fov": [-50f32, 50f32],
        "image_dimensions": [256u32, 256u32],
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
    })
    .to_string();

    // Act
    let response_zero = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_zero)
        .send()
        .await
        .expect("Failed to execute request.");
    let response_negative = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_negative)
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
    let body = json!({
        "fov": [50f32, 50f32],
        "image_dimensions": [256u32, 256u32],
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
    })
    .to_string();

    // Act
    let response = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body)
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
    let body_negative = json!({
        "fov": [50f32, 50f32],
        "image_dimensions": [256u32, 256u32],
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
    })
    .to_string();
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
    })
    .to_string();

    // Act
    let response_negative = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_negative)
        .send()
        .await
        .expect("Failed to execute request.");
    let response_positive = client
        .post(&format!("{}/renders", &app_address))
        .header("Content-Type", "application/json")
        .body(body_positive)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(400, response_negative.status().as_u16());
    assert_eq!(400, response_positive.status().as_u16());
}
