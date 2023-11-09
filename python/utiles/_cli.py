"""Utiles cli"""
from __future__ import annotations

import logging
import sys

import click
from utiles import ut_cli, __version__

logger = logging.getLogger(__name__)


class NoHelpCommand(click.Command):
    def get_help_option(self, _ctx: click.Context) -> None:
        return None


# The CLI command group.
@click.command(
    name="utiles",
    cls=NoHelpCommand,
    help="utiles cli (python-rust)",
    no_args_is_help=False,
    context_settings={
        "ignore_unknown_options": True,
        "allow_extra_args": True,
    },
)
@click.version_option(version=__version__, message="%(version)s")
def cli() -> None:
    """Execute the main utiles command"""
    args = ["ut", *sys.argv[1:]]
    try:
        res = ut_cli(args)
        click.echo(res, err=True)
    except Exception as e:
        logger.error(e)
        raise click.BadParameter(str(e))


if __name__ == "__main__":
    cli()