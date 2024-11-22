from __future__ import annotations

import warnings

import pytest
from pytest_benchmark.fixture import BenchmarkFixture

import utiles


def test_check_build_profile() -> None:
    assert (
        utiles.__build_profile__ == "debug" or utiles.__build_profile__ == "release"
    ), f"utiles.__build_profile__ is not 'debug'/'release': {utiles.__build_profile__}"


def _warn_benchmarking_with_debug_build() -> None:
    warnings.warn("utiles is built in debug mode", UserWarning, stacklevel=2)


@pytest.mark.filterwarnings("ignore:.*PytestBenchmarkWarning*")
def test_benchmarking_with_debug_build_profile(benchmark: BenchmarkFixture) -> None:
    # warn that this is a debug build
    if not benchmark.disabled and utiles.__build_profile__ == "debug":
        _warn_benchmarking_with_debug_build()
    # stupid benchmark to silence pytest-benchmark warning about no benchmarks...
    benchmark(
        lambda: utiles.__build_profile__ == "debug"
        or utiles.__build_profile__ == "release",
    )
