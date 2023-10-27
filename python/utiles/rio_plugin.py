import click
from utiles.cli import cli as rio_utiles

__all__ = ("rio_ut", "rio_utiles")

rio_ut = click.CommandCollection(
    sources=[rio_utiles], name="ut", help="utiles cli (alias)"
)
