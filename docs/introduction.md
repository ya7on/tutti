# Introduction

Tutti is a lightweight CLI tool for orchestrating processes - run, coordinate, and monitor multiple local processes with ease.

## What is Tutti

Tutti helps developers manage complex local development environments where multiple services need to run simultaneously. Instead of opening multiple terminals and starting each service manually, you describe everything in a simple TOML configuration file and start them all with one command.

## Why Use Tutti

- **Simplified Development** - No need to remember complex startup sequences
- **Dependency Management** - Services start in the correct order based on dependencies
- **Unified Logging** - All service logs in one place with colored prefixes
- **Easy Environment Setup** - New team members can start the entire stack with one command
- **Lightweight Alternative** - No Docker overhead for local development

## How It Works

1. Create a `tutti.toml` configuration file describing your services
2. Run `tutti-cli run -f tutti.toml` to start all services
3. Monitor colored logs from all services in one terminal
4. Press Ctrl+C to gracefully stop all services

Tutti is perfect for microservice architectures, full-stack development, or any scenario where you need to coordinate multiple local processes.
