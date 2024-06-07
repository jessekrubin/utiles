from __future__ import annotations

import json
import os
import sys
from typing import Dict, Union

from utiles import _utiles
from utiles.__about__ import __pkgroot__, __title__, __version__


def _utiles_ext_info() -> Dict[str, Union[str, int]]:
    size = os.path.getsize(_utiles.__file__)
    return {
        "abspath": os.path.abspath(_utiles.__file__),
        "fsize": size,
        "fsize_str": _utiles.fmt_nbytes(size),
        "build_profile": str(_utiles.__build_profile__),
    }


def main() -> None:
    """Print package metadata"""

    sys.stdout.write(
        json.dumps(
            {
                "package": __title__,
                "version": __version__,
                "pkgroot": __pkgroot__,
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
