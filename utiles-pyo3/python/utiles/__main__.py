from __future__ import annotations

import json
import os
import sys

from utiles import _utiles
from utiles.__about__ import (
    __allocator__,
    __authors__,
    __build_profile__,
    __build_timestamp__,
    __git_repo__,
    __git_sha__,
    __opt_level__,
    __pkgroot__,
    __target__,
    __title__,
    __version__,
)


def _utiles_ext_info() -> dict[str, str | int]:
    size = os.path.getsize(_utiles.__file__)
    return {
        "abspath": os.path.abspath(_utiles.__file__),
        "allocator": __allocator__,
        "build_profile": __build_profile__,
        "build_timestamp": __build_timestamp__,
        "fsize": size,
        "fsize_str": _utiles.fmt_nbytes(size),
        "opt-level": __opt_level__,
        "target": __target__,
    }


def _lib_info() -> dict[str, str | int | dict[str, str | int]]:
    return {
        "package": __title__,
        "version": __version__,
        "authors": __authors__,
        "pkgroot": __pkgroot__,
        "git": {
            "sha": __git_sha__,
            "repo": __git_repo__,
        },
        "_utiles": _utiles_ext_info(),
    }


def main() -> None:
    sys.stdout.write(json.dumps(_lib_info(), indent=2))


if __name__ == "__main__":
    if sys.argv[-1].endswith("__main__.py"):
        main()
    else:
        from utiles.cli import cli

        cli()
