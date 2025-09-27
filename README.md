# tutti
Lightweight CLI tool for orchestrating processes – run, coordinate, and monitor multiple local processes with ease.

## Usage

1. Create a configuration file (e.g., `tutti.yaml`) with the following format:

  ```yaml
  services:
    - name: service1
      cmd: ["command1", "arg1", "arg2"]
    - name: service2
      cmd: ["command2", "arg1", "arg2"]
  ```
2. Run `tutti` with the configuration file:

  ```sh
  $ cargo run -- -f tutti.yaml
  ```
