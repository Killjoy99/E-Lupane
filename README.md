# `1. E-Lupane`

An E-Learning system build in rust with a python sdk and also supports customs pluggins from developers. But each developer gets assisgned a plugin signing key so that we validate and verify before integrating each plugin to the system.

## `1.1 Supported Plugin Languages`

As of now the supported plugin languages are `rust` and `python`. Next we will support structured php shared objects.
All plugins should run independantly as a fall back mechanism when one fails other components of the system keep moving.

## `1.2 Plugin structures`
Here is the plugin structure for each language as well as the functions a plugin should expose and subscribe to.

### `1.2.1 Rust`
This is the default and native plugin structure supported out of the box and does not rely on an external sdk.


### `1.2.2 Python`


### `1.2.3 WebAssembly`


### `1.2.4 PHP`


### `1.2.5 C#`


## `Plugins`

These run as isolated pieces over grpc and process isolation

## `Requirements`

Install redis

```sh

curl -fsSL https://packages.redis.io/gpg | sudo gpg --dearmor -o /usr/share/keyrings/redis-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/redis-archive-keyring.gpg] https://packages.redis.io/deb $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/redis.list
sudo apt-get update
sudo apt-get install redis
```

Start Redis

```sh

# Start PostgreSQL
sudo service postgresql start

# Start Redis
sudo service redis-server start
```

## `Database Setup`

E-Lupane uses **PostgreSQL** for its relational data and **Redis** for the
inter-plugin messaging backbone. On Windows these run inside **WSL** and are
reached from the Windows app through WSL's `localhost` port forwarding.

### `1. Provision with the installer`

The `installer` crate (`crates/installer`) installs and configures both
services. It is **idempotent** — safe to run repeatedly — and:

1. checks whether PostgreSQL and Redis are installed, running, and configured;
2. installs anything that is missing (`apt` inside the WSL distro);
3. starts the `postgresql` / `redis-server` services and enables them on boot;
4. creates the application role `elupane` (`LOGIN`, `CREATEDB`,
   password `elupane`) and the `elupane` database owned by it;
5. writes `DATABASE_URL` + `REDIS_URL` into the root `.env`.

```sh
cargo run -p installer              # checks + report
cargo run -p installer -- --setup   # full setup (install + configure)
cargo run -p installer -- --install     # install missing components only
cargo run -p installer -- --configure   # configure existing components only
cargo run -p installer -- --help        # full usage / options
```

The resulting connection strings:

```sh
DATABASE_URL=postgres://elupane:elupane@localhost:5432/elupane
REDIS_URL=redis://127.0.0.1:6379
```

### `2. Manual service management (inside WSL)`

```sh
wsl -u root -e bash -lc "systemctl status postgresql"     # PostgreSQL
wsl -u root -e bash -lc "systemctl status redis-server"   # Redis
# start / stop / enable on boot use the same pattern:
#   systemctl start|stop|enable|disable <service>
```

### `3. Application configuration`

The `.env` files already point at the `elupane` database:

* root `.env` — used by the backend gateway and plugins:
  `DATABASE_URL=postgres://elupane:elupane@localhost:5432/elupane`
* `plugins/frontend/.env` — used by the Dioxus fullstack server:
  `DATABASE_URL=postgres://elupane:elupane@localhost:5432/elupane?sslmode=disable`

The code defaults (`plugins/auth/src/main.rs`, `plugins/frontend/src/db.rs`)
match these credentials, so the system works out of the box.

## `Startup Flow`

How the system startsup during installation

1. `Check PostgreSQL`
2. `Check Redis`
3. `Ensure Database`
4. `Run migrations`
5. `Generate Config`

## `How to run`

```sh
# 1. One-time database setup (install Postgres + Redis, start services, create
#    elupane user + database, write .env) — see "Database Setup" above.
cargo run -p installer -- --setup

# 2. Run the backend (gateway supervises all plugins, including auth).
cargo run -p gateway

# Or the full stack via serve.sh:
./scripts/serve.sh --prod     # production: gateway launches plugins + frontend
./scripts/serve.sh            # dev: gateway + dx serve (hot-reload)
```

## WSL PROXY
```sh
export http_proxy=10.76.156.214:8080
export https_proxy=10.76.156.214:8080
```
