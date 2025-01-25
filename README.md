# Telegram Mini Apps Init Data Parser for Rust

[![Crates.io](https://img.shields.io/crates/v/init-data-rs.svg)](https://crates.io/crates/init-data-rs)
[![Documentation](https://docs.rs/init-data-rs/badge.svg)](https://docs.rs/init-data-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/escwxyz/init-data-rs/workflows/Tests/badge.svg)](https://github.com/escwxyz/init-data-rs/actions)
[![codecov](https://codecov.io/gh/escwxyz/init-data-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/escwxyz/init-data-rs)

A Rust library for parsing and validating Telegram Mini Apps init data. This library helps you work with the data passed from Telegram to your Mini App, ensuring its authenticity and integrity.

## Features

- Parse init data from query string format
- Validate init data signature using bot token
- Support for third-party bot validation
- Type-safe data structures
- Comprehensive error handling
- 100% test coverage

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
init-data-rs = "0.1.0"
```

## Usage

```rust
use init_data_rs::{validate, InitData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let init_data = "query_id=AAHdF6IQAAAAAN0XohDhrOrc&user=%7B%22id%22%3A279058397%7D&auth_date=1662771648&hash=...";
    let bot_token = "YOUR_BOT_TOKEN";

    // Validate and parse init data
    let data: InitData = validate(init_data, bot_token, None)?;

    // Access parsed data
    if let Some(user) = data.user {
        println!("User ID: {}", user.id.0);
    }

    Ok(())
}
```

### Third-party Bot Validation

```rust
use init_data_rs::validate_third_party;

let data = validate_third_party(init_data, bot_token, third_party_token, None)?;
```

## Documentation

For detailed documentation, visit [docs.rs/init-data-rs](https://docs.rs/init-data-rs).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

This is a Rust port of the official [Golang implementation](https://github.com/Telegram-Mini-Apps/init-data-golang).
