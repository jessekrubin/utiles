"""Nox sessions for linting, docs, and testing."""
from __future__ import annotations

import argparse
import os
import shutil
from pathlib import Path

import nox

DIR = Path(__file__).parent.resolve()

nox.options.sessions = ["test"]


@nox.session
def test(session: nox.Session) -> None:
    """Run the unit and regular tests."""
    session.install("maturin")
    session.install("pytest", "hypothesis", "pytest-cov", "pytest-benchmark")
    session.run("maturin", "develop", "--release", "--extras=test")
    session.run("pytest")
