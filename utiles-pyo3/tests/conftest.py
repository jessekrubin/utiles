from pathlib import Path
from typing import Union

import pytest

PWD = Path(__file__).parent

_REPO_ROOT: Union[Path, None] = None


def _repo_root() -> Path:
    global _REPO_ROOT
    if _REPO_ROOT is not None:
        return _REPO_ROOT
    _root = PWD
    for _i in range(5):
        _root = _root.parent
        if (_root / ".github").is_dir():
            _REPO_ROOT = _root
            return _root
    raise RuntimeError("Could not find repo root")


@pytest.fixture
def repo_root() -> Path:
    return _repo_root()


@pytest.fixture
def test_data_root(repo_root: Path) -> Path:
    return repo_root / "test-data"
