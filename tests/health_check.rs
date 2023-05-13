use std::net::TcpListener;

use serde_json::json;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use space_telescope::configuration::{get_configuration, DatabaseSettings};
use space_telescope::startup::run;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

/// Spin up instance of our application
/// and returns its address (i.e. http://localhost:XXXX)
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TcpListener.");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configuration.database).await;

    let server = run(listener, db_pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);

    TestApp { address, db_pool }
}

pub async fn configure_database(db_config: &DatabaseSettings) -> PgPool {
    let mut db_connection = PgConnection::connect(&db_config.connection_string_instance())
        .await
        .expect("Failed to connect to Postgres.");

    db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &db_config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let db_pool = PgPool::connect(&db_config.connection_string_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to migrate the database.");

    db_pool
}

#[tokio::test]
async fn test_health_check_success() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &test_app.address))
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
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane": {
            "basis": [
                [1f32, 0f32, 0f32],
                [0f32, 1f32, 0f32]
            ]
        },
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
        .post(&format!("{}/renders", &test_app.address))
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(202, response.status().as_u16());

    let render = sqlx::query!("SELECT * FROM renders")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch queued render.");

    assert_eq!(render.image_url, None);
    assert_eq!(render.fov_x, body["fov"][0]);
    assert_eq!(render.fov_y, body["fov"][1]);
    assert_eq!(render.image_dimension_x, body["image_dimensions"][0]);
    assert_eq!(render.image_dimension_y, body["image_dimensions"][1]);
    assert_eq!(
        render.fundamental_plane_basis_vector_1,
        vec![1f32, 0f32, 0f32]
    );
    assert_eq!(
        render.fundamental_plane_basis_vector_2,
        vec![0f32, 1f32, 0f32]
    );
    assert_eq!(render.observer_position, vec![0f32, 0f32, 0f32]);
    assert_eq!(render.latitude, body["latitude"]);
    assert_eq!(render.longitude, body["longitude"]);
    assert_eq!(
        render.broadband_filters,
        vec![
            "SDSS_U".to_string(),
            "SDSS_G".to_string(),
            "SDSS_R".to_string()
        ]
    );
    assert_eq!(render.narrowband_filters, vec![0.55555f32]);
}

#[tokio::test]
async fn test_post_renders_returns_400_for_missing_body_fields() {
    // Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane": {
            "basis": [
                [1f32, 0f32, 0f32],
                [0f32, 1f32, 0f32]
            ]
        },
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
        .post(&format!("{}/renders", &test_app.address))
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
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body_zero = json!({
        "fov": [50f32, 0f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane": {
            "basis": [
                [1f32, 0f32, 0f32],
                [0f32, 1f32, 0f32]
            ]
        },
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
        "fundamental_plane": {
            "basis": [
                [1f32, 0f32, 0f32],
                [0f32, 1f32, 0f32]
            ]
        },
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
        .post(&format!("{}/renders", &test_app.address))
        .header("Content-Type", "application/json")
        .body(body_zero.to_string())
        .send()
        .await
        .expect("Failed to execute request.");
    let response_negative = client
        .post(&format!("{}/renders", &test_app.address))
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
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane": {
            "basis": [
                [1f32, 0f32, 0f32],
                [2f32, 0f32, 0f32]
            ]
        },
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
        .post(&format!("{}/renders", &test_app.address))
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
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body_negative = json!({
        "fov": [50f32, 51f32],
        "image_dimensions": [256u32, 257u32],
        "fundamental_plane": {
            "basis": [
                [1f32, 0f32, 0f32],
                [0f32, 1f32, 0f32]
            ]
        },
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
        "fundamental_plane": {
            "basis": [
                [1f32, 0f32, 0f32],
                [0f32, 1f32, 0f32]
            ]
        },
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
        .post(&format!("{}/renders", &test_app.address))
        .header("Content-Type", "application/json")
        .body(body_negative.to_string())
        .send()
        .await
        .expect("Failed to execute request.");
    let response_positive = client
        .post(&format!("{}/renders", &test_app.address))
        .header("Content-Type", "application/json")
        .body(body_positive.to_string())
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(400, response_negative.status().as_u16());
    assert_eq!(400, response_positive.status().as_u16());
}
