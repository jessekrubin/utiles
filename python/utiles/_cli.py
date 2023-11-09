"""Utiles cli"""
from __future__ import annotations

import logging
import sys

from utiles import ut_cli

logger = logging.getLogger(__name__)


def cli() -> None:
    args = ["ut", *sys.argv[1:]]
    try:
        ut_cli(args)
    except Exception as e:
        logger.error(e)
        raise e from e


if __name__ == "__main__":
    cli()
