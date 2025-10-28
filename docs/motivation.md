# Motivation

Tutti was created to solve a very common problem for developers who work with multiple local services. When working on modern backend systems, it is normal to have several services that depend on each other. For example, one service might need a database, another might need that service, and so on. This often means opening several terminal tabs and manually running each command in the right order. It is not hard, but it is repetitive and annoying.

Many people solve this with Docker and docker-compose. Docker lets you define dependencies and start everything with one command. But for many developers, especially on macOS, Docker builds are slow. Even a small code change can trigger a long rebuild. That slows down development a lot. In some cases, cross-compiling or mounting volumes is an option, but it adds complexity and is not always practical. If the services are already runnable locally, Docker becomes unnecessary overhead.

Tutti is designed to remove that overhead. It lets developers define services in a simple TOML config file: the command to run, working directory, environment variables, and dependencies. Tutti then starts the services in the correct order, streams all logs to one place, and cleanly shuts everything down with a single Ctrl+C.

The goal is not to replace Docker in production. The goal is to make local development faster and simpler. Tutti gives developers a single command to bring up their entire local stack without containers, without long build times, and without juggling multiple terminals. It is like docker-compose, but for real processes on your machine.
