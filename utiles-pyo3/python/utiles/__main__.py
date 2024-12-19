from __future__ import annotations

import json
import os
import sys

from utiles import _utiles
from utiles.__about__ import __authors__, __pkgroot__, __title__, __version__


def _utiles_ext_info() -> dict[str, str | int]:
    size = os.path.getsize(_utiles.__file__)
    return {
        "abspath": os.path.abspath(_utiles.__file__),
        "fsize": size,
        "fsize_str": _utiles.fmt_nbytes(size),
        "build_profile": _utiles.__build_profile__,
        "build_timestamp": _utiles.__build_timestamp__,
    }


def main() -> None:
    """Print package metadata"""

    sys.stdout.write(
        json.dumps(
            {
                "package": __title__,
                "version": __version__,
                "pkgroot": __pkgroot__,
                "authors": __authors__,
                "website": "https://github.com/jessekrubin/utiles",
                "_utiles": _utiles_ext_info(),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    if sys.argv[-1].endswith("__main__.py"):
        main()
    else:
        from utiles.cli import cli

        cli()
