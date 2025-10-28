# tutti

[![Crates.io Version](https://img.shields.io/crates/v/tutti-cli)](https://crates.io/crates/tutti-cli)
[![GitHub License](https://img.shields.io/github/license/ya7on/tutti)](LICENSE)
[![CI](https://github.com/ya7on/tutti/actions/workflows/rust.yml/badge.svg)](https://github.com/ya7on/tutti/actions/workflows/rust.yml)
[![Docs](https://img.shields.io/github/actions/workflow/status/ya7on/tutti/docs.yml?label=docs)](https://ya7on.github.io/tutti)
[![codecov](https://codecov.io/gh/ya7on/tutti/graph/badge.svg?token=UCYX4KOI0F)](https://codecov.io/gh/ya7on/tutti)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/ya7on/tutti)

A lightweight CLI tool for orchestrating local processes. Start multiple services with one command, handle dependencies automatically, and see all logs in one place.

## Quick Start

Install:
```bash
cargo install tutti-cli
```

Create a `tutti.toml` config:
```toml
version = 1

[services.api]
cmd = ["python", "app.py"]
env = { PORT = "3000" }

[services.frontend]
cmd = ["npm", "start"]
deps = ["api"]
cwd = "./frontend"
```

Run:
```bash
tutti-cli run -f tutti.toml
```

## What is Tutti

Tutti solves the common developer problem of managing multiple local services. Instead of opening several terminals and remembering which services to start in what order, you define everything in a simple config file.

## Installation

### From crates.io
```bash
cargo install tutti-cli
```

### From source
```bash
git clone https://github.com/ya7on/tutti
cd tutti
cargo build --release
# Binary will be at target/release/tutti-cli
```

## Basic Usage

**Start all services:**
```bash
tutti-cli run -f tutti.toml
```

**Start specific services (and their dependencies):**
```bash
tutti-cli run -f tutti.toml frontend api
```

**Example output:**
```
[database] Starting PostgreSQL on port 5432
[api] Server listening on http://localhost:3000
[frontend] Development server started on port 8080
```

## Configuration Format

Services are defined in TOML format:

```toml
version = 1

[services.database]
cmd = ["postgres", "-D", "./data"]

[services.api]
cmd = ["python", "server.py"]
deps = ["database"]
env = { DATABASE_URL = "postgresql://localhost/mydb" }
cwd = "./backend"
restart = "always"

[services.frontend]
cmd = ["npm", "run", "dev"]
deps = ["api"]
cwd = "./frontend"
```

**Configuration options:**
- `cmd` (required) - Command and arguments to run
- `deps` (optional) - List of service dependencies
- `env` (optional) - Environment variables
- `cwd` (optional) - Working directory
- `restart` (optional) - Restart policy (default: "never")

## Documentation

Full documentation with examples and advanced configuration options:

**https://ya7on.github.io/tutti**

## License

Licensed under [MIT License](LICENSE)
