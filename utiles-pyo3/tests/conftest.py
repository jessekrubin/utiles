from functools import lru_cache
from pathlib import Path

import pytest

PWD = Path(__file__).parent


@lru_cache(maxsize=1)
def _repo_root() -> Path:
    _root = PWD
    for _i in range(5):
        _root = _root.parent
        if (_root / ".github").is_dir():
            return _root
    ex = RuntimeError("Could not find repo root")
    raise ex


@pytest.fixture
def repo_root() -> Path:
    return _repo_root()


@pytest.fixture
def test_data_root(repo_root: Path) -> Path:
    return repo_root / "test-data"


@pytest.fixture(params=["flat", "hash", "norm"])
def db_type(request: pytest.FixtureRequest) -> str:
    """Fixture for testing different db/schema types"""
    return str(request.param)
