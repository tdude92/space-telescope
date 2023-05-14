use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

use space_telescope::configuration::get_configuration;
use space_telescope::startup::run;
use space_telescope::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Setup tracing
    let subscriber = get_subscriber("space-telescope", "info", std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let db_pool = PgPool::connect_lazy(
        &configuration
            .database
            .connection_string_db()
            .expose_secret(),
    )
    .expect("Failed to create Postgres connection pool.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool)?.await
}
