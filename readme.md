# Reslab Product Microservice

## CRUD REST API with Rust and MySQL using Axum & SQLx

### Current Stack Version

- Rust 1.81.0

### Install

```sh
# Build & Run Project
cargo build
cargo run

# CLI For Watch source when running & Automatically rebuild the project
cargo install cargo-watch

# Run with watch
cargo watch -q -c -w src/ -x run


# Docker Compose up & detach
docker-compose up -d

# Shutdown docker compose
docker-compose down


# CLI For migration
cargo install sqlx-cli

# create a migration
sqlx migrate add -r create_products_table

# perform migration up
sqlx migrate run

# perform migration down/revert (optional)
sqlx migrate revert
```
