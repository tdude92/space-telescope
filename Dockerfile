FROM rust:1.69.0

ENV SQLX_OFFLINE true
ENV APP_ENVIRONMENT production

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN cargo build --release
ENTRYPOINT ["./target/release/space-telescope"]
