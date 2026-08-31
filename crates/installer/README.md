# `Installer Crate`

This is the installation helper for the E-Lupane system. It provisions the two
system services — **PostgreSQL** and **Redis** — that the rest of the platform
depends on.

On Windows these services run inside **WSL**; every command the installer issues
is dispatched into the WSL distro (as root) automatically. On Linux the commands
run directly, elevated with `sudo` when needed.

```sh
cargo run -p installer              # checks + report
cargo run -p installer -- --setup   # full setup (install + configure)
cargo run -p installer -- --install     # install missing components only
cargo run -p installer -- --configure   # configure existing components only
cargo run -p installer -- --help        # full usage / options
```

## What `--setup` does (idempotent)

1. **Checks** whether PostgreSQL and Redis are installed, running, and
   configured (including the `elupane` role + database).
2. **Installs** anything missing — `apt` packages inside the WSL distro.
3. **Configures**:
   - PostgreSQL: start the cluster, enable on boot, create role
     `elupane` (`LOGIN`, `CREATEDB`, password `elupane`) and database
     `elupane` owned by it.
   - Redis: start the service, enable on boot.
4. **Writes** `DATABASE_URL` + `REDIS_URL` into the root `.env`.

Defaults (`elupane` / `elupane` / `elupane` / `5432`) can be overridden via the
`DB_USER`, `DB_PASSWORD`, `DB_NAME`, and `DB_PORT` environment variables.

## Layout

```
src/
├── main.rs             CLI entry point (--check / --install / --configure / --setup)
├── bootstrap.rs        orchestration of the install + configure flow
├── wsl.rs              bridge: run shell commands in WSL (Windows) or locally (Linux)
├── checks/             readiness checks (postgres, redis, os)
├── installers/         install + configure actions (postgres, redis)
├── config/generate.rs  writes DATABASE_URL + REDIS_URL into .env
└── report.rs           human-readable check report
```

