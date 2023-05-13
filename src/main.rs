use std::net::TcpListener;

use env_logger::Env;
use sqlx::PgPool;

use space_telescope::configuration::get_configuration;
use space_telescope::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let configuration = get_configuration().expect("Failed to read configuration.");

    let db_pool = PgPool::connect(&configuration.database.connection_string_db())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("127.0.0.1:{}", configuration.port);
    let listener = TcpListener::bind(address)?;
    run(listener, db_pool)?.await
}
