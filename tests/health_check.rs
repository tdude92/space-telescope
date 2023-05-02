use std::net::TcpListener;

#[tokio::test]
async fn test_health_check_success() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind TcpListener.");
    let port = listener.local_addr().unwrap().port();
    let server = space_telescope::run(listener).expect("Failed to bind address");

    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
