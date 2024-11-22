from __future__ import annotations

from collections import Counter

import pytest

import utiles
from utiles import _utiles as libutiles


def test_import() -> None:
    assert utiles
    assert libutiles


@pytest.mark.skip(reason="ruff now handles this")
def test_all_sorted() -> None:
    all_current = utiles.__all__
    all_tuple = tuple(sorted(utiles.__all__))
    assert all_current == all_tuple, f"{all_current} != {all_tuple}"


def test_all_no_duplicates() -> None:
    c = Counter(utiles.__all__)
    duplicates = [k for k, v in c.items() if v > 1]
    assert not duplicates, f"duplicates: {duplicates}"


def test_missing_from_libutiles() -> None:
    tmp_ignored = {"debug", "error", "info", "trace", "warn", "lager", "Lager"}
    ignored_members = {
        *tmp_ignored,
        "__all__",
        "__doc__",
        "__file__",
        "__loader__",
        "__name__",
        "__package__",
        "__spec__",
    }
    libutiles_members = set(dir(libutiles)) - ignored_members
    utiles_all = set(utiles.__all__) - ignored_members
    missing = {el for el in libutiles_members if el not in utiles_all} - ignored_members
    all_tuple = tuple(
        sorted(
            {
                *utiles_all,
                *missing,
            }
        )
    )

    "\n".join([f"from utiles.libutiles import {el}" for el in all_tuple])
    if missing:
        msg = f"Missing from libutiles: {missing}"
        raise AssertionError(msg)
    # print(libutiles_imports_str)


def main() -> None:
    ignored_members = {
        "__all__",
        "__doc__",
        "__file__",
        "__loader__",
        "__name__",
        "__package__",
        "__spec__",
    }
    libutiles_members = set(dir(libutiles)) - ignored_members

    utiles_all = set(utiles.__all__) - ignored_members

    missing = {el for el in libutiles_members if el not in utiles_all} - ignored_members
    all_tuple = tuple(
        sorted(
            {
                *utiles_all,
                *missing,
            }
        )
    )

    libutiles_imports_str = "\n".join(
        [f"from utiles.libutiles import {el}" for el in all_tuple]
    )
    if missing:
        print("Missing from libutiles:")  # noqa: T201
        print(all_tuple)  # noqa: T201
    print(utiles.__all__)  # noqa: T201
    print(libutiles_imports_str)  # noqa: T201


if __name__ == "__main__":
    main()
