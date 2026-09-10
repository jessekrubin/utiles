#!/usr/bin/env bash

uv run maturin develop -m utiles-pyo3/Cargo.toml
# uv pip install mypy==1.19.1
uv pip install -U mypy
uv run python -m mypy.stubtest --version
uv run python -m mypy.stubtest \
  --mypy-config-file utiles-pyo3/pyproject.toml \
  --ignore-disjoint-bases \
  --concise \
  utiles
