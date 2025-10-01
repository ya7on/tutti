# tutti-cli

[![Crates.io Version](https://img.shields.io/crates/v/tutti-cli)](https://crates.io/crates/tutti-cli)
[![GitHub License](https://img.shields.io/github/license/ya7on/tutti)](LICENSE)

`tutti-cli` is the command line interface for **tutti**, a lightweight tool to orchestrate local processes and microservices.
It allows you to define services in a simple config file and start them all with one command.

## Installation

From crates.io:

```bash
cargo install tutti-cli
```

From source:

```bash
git clone https://github.com/ya7on/tutti
cd tutti/crates/tutti-cli
cargo build --release
# binary will be at target/release/tutti-cli
```

## Quick Start

Create a `tutti.toml`:

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

## Documentation

[![Documentation](https://img.shields.io/badge/documentation-yes-brightgreen)](https://ya7on.github.io/tutti)
