dev: develop test

develop:
    maturin develop -m utiles-pyo3/Cargo.toml

cargo-test:
    cargo test

build: cargo-test
    maturin build

build-release:
    maturin build --release

dev-rel:
    maturin develop --release

test:
    cd utiles-pyo3 && pytest --benchmark-skip

test-release: build-release
    cd utiles-pyo3 && pytest --benchmark-skip

bench: build-release
    pytest -vv

cargo-fmt:
    cargo fmt

sort-all:
    sort-all utiles-pyo3/python/utiles/__init__.py

black:
    black utiles-pyo3

fmt: cargo-fmt black

mypy:
    mypy utiles-pyo3/python/utiles utiles-pyo3/tests

ruff:
    ruff .

ruffix:
    ruff --fix --show-fixes

clippy:
    cargo clippy

lintpy: ruff mypy

lintrs: clippy

lint: lintpy lintrs


