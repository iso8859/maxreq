# Rust Project Setup Instructions

## Prerequisites
- Rust toolchain (rustc, cargo) is now installed via rustup.
- If you open a new terminal, run:
  
  ```bash
  source "$HOME/.cargo/env"
  ```
  to ensure cargo and rustc are in your PATH.

## Building and Running

### rust-mini
```
cd rust/rust-mini
cargo build --release
cargo run --release --bin rust-mini
```

### user-token-api
```
cd rust/user-token-api
cargo build --release
cargo run --release --bin user-token-api
```

### user-token-api-actix
```
cd rust/user-token-api-actix
cargo build --release
cargo run --release
```

## Notes
- All dependencies are managed in each `Cargo.toml`.
- For more info: https://www.rust-lang.org/learn
