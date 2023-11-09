"""Package metadata/info"""

from .libutiles import __version_lib__  # noqa: TID252

__all__ = ("__title__", "__description__", "__pkgroot__", "__version__")
__title__ = "utiles"
__description__ = "utiles = utils + tiles + rust"
__pkgroot__ = __file__.replace("__about__.py", "").rstrip("/\\")
__version__ = __version_lib__
