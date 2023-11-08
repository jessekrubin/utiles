"""Utiles cli"""
from __future__ import annotations

import logging
import sys

import click

import utiles
from utiles import libutiles

logger = logging.getLogger(__name__)


class NoHelpCommand(click.Command):
    def get_help_option(self, ctx: click.Context) -> None:
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
# @click.argument("cmd", required=false)
# @click.option("--verbose", "-v", count=True, help="Increase verbosity.")
# @click.option("--quiet", "-q", count=True, help="Decrease verbosity.")
@click.version_option(version=utiles.__version__, message="%(version)s")
@click.pass_context
def cli(ctx: click.Context) -> None:
    """Execute the main utiles command"""
    # verbosity = verbose - quiet
    # configure_logging(verbosity)
    # ctx.obj["verbosity"] = verbosity
    args = ["ut", *sys.argv[1:]]
    try:
        res = libutiles.ut_cli(args)
        click.echo(res, err=True)
    except Exception as e:
        logger.error(e)
        raise click.BadParameter(str(e))


if __name__ == "__main__":
    cli()
