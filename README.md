# tutti
Lightweight CLI tool for orchestrating processes – run, coordinate, and monitor multiple local processes with ease.

## Installation

You can install `tutti-cli` using Cargo:

```sh
$ cargo install tutti-cli
```

## Usage

1. Create a configuration file (e.g., `tutti.toml`) with the following format:

  ```toml
  version = 1

  [services.service1]
  cmd = ["command1", "arg1", "arg2"]
  env = { KEY1 = "VALUE1", KEY2 = "VALUE2" }
  deps = ["service2"]

  [services.service2]
  cmd = ["command2", "arg1", "arg2"]
  [services.service2.env]
  KEY3 = "VALUE3"
  KEY4 = "VALUE4"
  ```
2. Run `tutti` with the configuration file:

  ```sh
  $ tutti-cli run -f tutti.toml
  ```
