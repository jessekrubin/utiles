pyut := "utiles-pyo3"
pyut_manifest := pyut / "Cargo.toml"
pyut_pyproject_toml := pyut / "pyproject.toml"

dev: develop test

develop:
    cd {{pyut}}
    maturin develop -m {{pyut_manifest}}

cargo-test:
    cargo test

build: cargo-test
    cd {{pyut}}
    maturin build -m {{pyut_manifest}}

build-release:
    cd {{pyut}}
    maturin build --release  -m {{pyut_manifest}}

dev-rel:
    cd {{pyut}}
    maturin develop --release -m {{pyut_manifest}}

test:
    cd {{pyut}}
    pytest --benchmark-disable --config-file={{pyut_pyproject_toml}} {{pyut}}

test-release: build-release
    cd {{pyut}}
    pytest --benchmark-disable --config-file={{pyut_pyproject_toml}} {{pyut}}

bench: dev-rel
    cd {{pyut}}
    pytest -vv --benchmark-only --config-file={{pyut_pyproject_toml}} {{pyut}}

cargo-fmt:
    cargo fmt

sort-all:
    sort-all {{pyut}}/python/utiles/__init__.py

black:
    black {{pyut}}/python {{pyut}}/tests

# format python
fmtpy:
    ruff format
    ruff check --select "I" --show-fixes --fix .

# format-check
fmtcpy:
    ruff format --check
    ruff check --select "I" --show-fixes .

fmt: cargo-fmt fmtpy

mypy:
    mypy --config-file {{pyut}}/pyproject.toml {{pyut}}/python {{pyut}}/tests

ruff:
    ruff check .

ruffix:
    ruff check . --fix --show-fixes

clippy:
    cargo clippy

lintpy: ruff mypy

lintrs: clippy

lint: lintpy lintrs

ci:
    cargo fmt -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

pipsync:
    uv pip sync ./utiles-pyo3/requirements/dev.txt
