# Configuration

Tutti uses TOML files to configure services. By default, it looks for a `tutti.toml` file in the current directory.

## Configuration Structure

### Root Level

```toml
version = 1

[services]
# Services are defined here
```

#### Root Parameters

- `version` (optional, defaults to `1`) - Configuration format version

### Services

Each service is described in a `[services.service_name]` section:

```toml
[services.my_service]
cmd = ["command", "arg1", "arg2"]
cwd = "/path/to/working/directory"
env = { VAR1 = "value1", VAR2 = "value2" }
deps = ["other_service"]
```

#### Service Parameters

- `cmd` (required) - Array of strings with command and arguments to run the service
- `cwd` (optional) - Working directory for the command execution
- `env` (optional) - Environment variables for the service
- `deps` (optional) - List of dependencies - names of other services that must be started before this one

#### Parameter Requirements

- `cmd` cannot be an empty array
- `cmd` cannot contain empty strings
- `deps` can only contain names of existing services

## Environment Variables

Environment variables can be defined in two ways:

### Inline Object

```toml
[services.api]
cmd = ["node", "server.js"]
env = { NODE_ENV = "development", PORT = "3000" }
```

### Separate Section

```toml
[services.api]
cmd = ["node", "server.js"]

[services.api.env]
NODE_ENV = "development"
PORT = "3000"
```

## Configuration Examples

### Simple Service

```toml
version = 1

[services.hello]
cmd = ["echo", "Hello World"]
```

### Service with Environment Variables

```toml
version = 1

[services.web]
cmd = ["python", "-m", "http.server"]
env = { PORT = "8000" }
cwd = "./public"
```

### Services with Dependencies

```toml
version = 1

[services.database]
cmd = ["postgres", "-D", "./data"]

[services.api]
cmd = ["node", "api.js"]
deps = ["database"]
env = { DB_HOST = "localhost" }

[services.frontend]
cmd = ["npm", "start"]
deps = ["api"]
cwd = "./frontend"
```

In this example, services will start in order: `database` → `api` → `frontend`.

### Complex Multi-Service Setup

```toml
version = 1

[services.redis]
cmd = ["redis-server", "--port", "6379"]

[services.postgres]
cmd = ["postgres", "-D", "./pgdata", "-p", "5432"]

[services.api-auth]
cmd = ["python", "-m", "auth_service"]
deps = ["postgres", "redis"]
cwd = "./services/auth"
[services.api-auth.env]
DATABASE_URL = "postgresql://localhost:5432/auth"
REDIS_URL = "redis://localhost:6379"

[services.api-users]
cmd = ["node", "users-service.js"]
deps = ["postgres"]
cwd = "./services/users"
[services.api-users.env]
DB_HOST = "localhost"
DB_PORT = "5432"

[services.api-gateway]
cmd = ["./api-gateway"]
deps = ["api-auth", "api-users"]
[services.api-gateway.env]
AUTH_SERVICE = "http://localhost:8001"
USERS_SERVICE = "http://localhost:8002"

[services.frontend]
cmd = ["npm", "run", "dev"]
deps = ["api-gateway"]
cwd = "./frontend"
env = { REACT_APP_API_URL = "http://localhost:8000" }
```
