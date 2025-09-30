# Usage

## Basic Commands

### Run All Services

```bash
tutti-cli run --file tutti.toml
```

### Run Specific Services

You can specify which services to start:

```bash
tutti-cli run --file tutti.toml service1 service2
```

When you specify services, all their dependencies will be automatically started as well.

## Command Options

### `tutti-cli run`

Starts services defined in the configuration file.

**Options:**
- `--file` / `-f` (required) - Path to the TOML configuration file
- `services` (optional) - List of service names to start

**Examples:**
```bash
# Start all services
tutti-cli run -f tutti.toml

# Start specific services
tutti-cli run -f config.toml api database

# Using long form
tutti-cli run --file ./config/tutti.toml frontend
```

## Process Management

Press `Ctrl+C` to stop all services gracefully

## Log Output

All service logs are combined into a single output stream with prefixes:

```
[database] Starting PostgreSQL on port 5432
[api] Server listening on http://localhost:3000
[frontend] Development server started on port 8080
[database] Ready to accept connections
```
