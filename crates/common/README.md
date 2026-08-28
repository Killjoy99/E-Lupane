# `Common Crate - Rust to Python Bindings`
To build the crates here we install maturin and build the python module and install it in our python venv

## `Installing uv`

```sh
curl -LsSf https://astral.sh/uv/install.sh | sh
```

## `Creating and actiavting a venv`

```sh
uv venv ~/.venv
source ~/.venv/bin/activate
```

## `Installing maturin`

```sh
uv pip install maturin
```

## `Building this as a python module`

```sh
maturin develop --release
```