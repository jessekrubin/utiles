"""Nox sessions for linting, docs, and testing."""
from __future__ import annotations

from pathlib import Path

import nox

DIR = Path(__file__).parent.resolve()

nox.options.sessions = ["test"]


def _session_install_test_deps(session: nox.Session) -> None:
    session.install("pytest", "hypothesis", "pytest-cov", "pytest-benchmark", "tomli")


@nox.session
def test(session: nox.Session) -> None:
    """Run the unit and regular tests."""
    session.install("maturin")
    _session_install_test_deps(session)
    session.run("maturin", "develop", "--release", "--extras=test")
    session.run("pytest")


@nox.session
def test_wheel(session: nox.Session) -> None:
    """Run the unit and regular tests."""
    # install from dist...
    session.install(
        "utiles",
    )
    session.install("maturin")
    _session_install_test_deps(session)
    session.run("maturin", "build", "--release", "--extras=test")
    session.run("pytest")


def _session_build_release(session: nox.Session) -> None:
    session.install("maturin")
    session.run("maturin", "build", "--release", "--strip")


@nox.session(
    name="build-release",
)
def build_release(session: nox.Session) -> None:
    """Build the release."""
    _session_build_release(session)


@nox.session
def bench(session: nox.Session) -> None:
    """Run the benchmarks."""
    session.install("maturin")
    _session_install_test_deps(session)
    _session_build_release(session)

    # install from wheel
    session.install(
        "utiles",
        "--find-links",
        "target/wheels",
        "--no-index",
        "--force-reinstall",
        "--no-deps",
    )
    session.install("click", "mercantile", "pmtiles")
    session.run("pytest")

    session.run("pytest", "bench")
