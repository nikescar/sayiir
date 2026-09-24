# sayiir-diesel

Diesel persistence backend for Sayiir workflow engine.

Supports SQLite, PostgreSQL, and MySQL via feature flags.

## Features

- `sqlite` - SQLite backend
- `postgres` - PostgreSQL backend
- `mysql` - MySQL backend

## Usage

```rust
use sayiir_diesel::DieselBackend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = DieselBackend::new("sqlite::memory:").await?;
    Ok(())
}
```

## License

MIT
