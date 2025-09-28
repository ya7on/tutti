# tutti
Lightweight CLI tool for orchestrating processes – run, coordinate, and monitor multiple local processes with ease.

## Usage

1. Create a configuration file (e.g., `tutti.toml`) with the following format:

  ```toml
  version = 1

  [services.service1]
  cmd = ["command1", "arg1", "arg2"]
  deps = ["service2"]

  [services.service2]
  cmd = ["command2", "arg1", "arg2"]
  ```
2. Run `tutti` with the configuration file:

  ```sh
  $ cargo run -- -f tutti.toml
  ```
