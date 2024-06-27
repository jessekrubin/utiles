"""Utiles cli"""

from __future__ import annotations

import logging
import sys

from utiles import ut_cli

logger = logging.getLogger(__name__)


def cli(
    args: list[str | int | float | bool | None] | None = None,
) -> None:
    _args = args or sys.argv[1:]

    # if first arg is "utiles" then remove it
    if _args and (_args[0] == "utiles" or _args[0] == "ut"):
        _args = _args[1:]

    try:
        ut_cli(["ut", *map(str, _args)])
    except Exception as e:
        logger.error(e)
        raise e from e


if __name__ == "__main__":
    cli()
