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

## `Startup Flow`

How the system startsup during installation

1. `Check PostgreSQL`
2. `Check Redis`
3. `Ensure Database`
4. `Run migrations`
5. `Generate Config`

## `How to run`

```sh
cargo run -p gateway # to run the backend.
```