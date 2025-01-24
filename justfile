#!/usr/bin/env just --justfile
# 'justfile'
# just-repo: https://github.com/casey/just
# just-docs: https://just.systems/man/en/

pyut := "utiles-pyo3"
pyut_manifest := pyut / "Cargo.toml"
pyut_pyproject_toml := pyut / "pyproject.toml"

@_default:
    just --list --unsorted

# main dev command
dev: fmt develop pytest cargo-test

# maturin develop
develop:
    cd {{ pyut }}
    maturin develop -m {{ pyut_manifest }}

# cargo test
cargo-test:
    cargo test

# maturin build
build: cargo-test
    cd {{ pyut }}
    maturin build -m {{ pyut_manifest }}

# maturin build --release
build-release:
    cd {{ pyut }}
    maturin build --release  -m {{ pyut_manifest }}

# maturin develop --release
dev-rel:
    cd {{ pyut }}
    maturin develop --release -m {{ pyut_manifest }}

# pytest
pytest:
    cd {{ pyut }}
    pytest --benchmark-disable -n 4 --config-file={{ pyut_pyproject_toml }} {{ pyut }}

# test release build python
test-release: build-release
    cd {{ pyut }}
    pytest --benchmark-disable --config-file={{ pyut_pyproject_toml }} {{ pyut }}

# run benchmarks via pytest-benchmark
bench: dev-rel
    cd {{ pyut }}
    pytest -vv --benchmark-only --config-file={{ pyut_pyproject_toml }} {{ pyut }}

cargo-fmt:
    cargo fmt

# sort imports
sort-all:
    sort-all {{ pyut }}/python/utiles/__init__.py

# format via black
black:
    black {{ pyut }}/python {{ pyut }}/tests

# format python
fmtpy:
    ruff format
    ruff check --select "I" --show-fixes --fix .

# format-check
fmtcpy:
    ruff format --check
    ruff check --select "I" --show-fixes .

# format rust and python
fmt: cargo-fmt fmtpy

# typecheck w/ mypy
mypy:
    mypy --config-file {{ pyut }}/pyproject.toml {{ pyut }}/python {{ pyut }}/tests

# ruff check/lint
ruff:
    ruff check .

# ruff check fix
ruffix:
    ruff check . --fix --show-fixes

# clippy lint
clippy:
    cargo clippy

# clippy lint fix
clippy-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings

# lint python
lintpy: ruff mypy

# lint rust
lintrs: clippy

# lint
lint: lintpy lintrs

# ci checks
ci:
    cargo fmt -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

check-features:
    cargo hack check --feature-powerset


pipsync:
    uv pip sync ./utiles-pyo3/requirements.dev.txt

# install utiles and utiles-oxipng
reinstall:
    cargo install --path ./crates/utiles
    cargo install --path ./crates/utiles-oxipng

# pip compile requirements
pip-compile:
    uv export > utiles-pyo3/requirements.dev.txt
    uv pip compile utiles-pyo3/requirements.dev.in -n > utiles-pyo3/requirements.dev.txt
