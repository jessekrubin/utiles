"""Utiles cli"""
from __future__ import annotations

import logging
import sys

from utiles import ut_cli

logger = logging.getLogger(__name__)


def cli() -> None:
    args = sys.argv[1:]

    # if first arg is "utiles" then remove it
    if args and (args[0] == "utiles" or args[0] == "ut"):
        args = args[1:]
    try:
        ut_cli(["ut", *args])
    except Exception as e:
        logger.error(e)
        raise e from e


if __name__ == "__main__":
    cli()
