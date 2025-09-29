# Usage

The main entry point is the `tutti run` command.
You specify a config file with services and their commands.

```bash
tutti run --file tutti.toml
```

If no file is given, Tutti looks for `tutti.toml`, `tutti-config.toml`, or `tutti.config.toml` in the current directory.
