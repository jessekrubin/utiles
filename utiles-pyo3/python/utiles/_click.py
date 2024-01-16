"""Utiles cli wrapped w/ click"""
from __future__ import annotations

import logging

from utiles import __version__
from utiles.cli import cli

try:
    import click
except ImportError as ie:
    msg = "click not installed for rio/legacy utiles cli: `pip install click`"
    raise ImportError(msg) from ie

logger = logging.getLogger(__name__)


class NoHelpCommand(click.Command):
    def get_help_option(self, _ctx: click.Context) -> None:
        return None


# The CLI command group.
def _click_cli(name: str) -> click.Command:
    @click.command(
        name=name,
        cls=NoHelpCommand,
        help="utiles cli (python-rust)",
        no_args_is_help=False,
        context_settings={
            "ignore_unknown_options": True,
            "allow_extra_args": True,
        },
    )
    @click.version_option(version=__version__, message="%(version)s")
    def _cli_fn() -> None:
        """Execute the main utiles command"""
        try:
            cli()
        except Exception as e:
            logger.error(e)
            raise click.BadParameter(str(e)) from e

    return _cli_fn


utiles_click = _click_cli("utiles")
ut_click = _click_cli("ut")

if __name__ == "__main__":
    utiles_click()
