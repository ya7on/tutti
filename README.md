# tutti

[![Crates.io Version](https://img.shields.io/crates/v/tutti-cli)](https://crates.io/crates/tutti-cli)
[![GitHub License](https://img.shields.io/github/license/ya7on/tutti)](LICENSE)
[![CI](https://github.com/ya7on/tutti/actions/workflows/rust.yml/badge.svg)](https://github.com/ya7on/tutti/actions/workflows/rust.yml)
[![Docs](https://img.shields.io/github/actions/workflow/status/ya7on/tutti/docs.yml?label=docs)](https://ya7on.github.io/tutti)
[![GitHub top language](https://img.shields.io/github/languages/top/ya7on/tutti)](README)
[![Crates.io Size](https://img.shields.io/crates/size/tutti-cli?label=binary%20size)](README)

Lightweight CLI tool for orchestrating processes – run, coordinate, and monitor multiple local processes with ease.

## Installation

You can install `tutti-cli` using Cargo:

```sh
$ cargo install tutti-cli
```

## Documentation

Documentation is available at https://ya7on.github.io/tutti

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
