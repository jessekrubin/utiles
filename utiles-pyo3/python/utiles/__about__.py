"""Package metadata/info"""

from __future__ import annotations

from utiles._utiles import (
    __authors__,
    __build_profile__,
    __build_timestamp__,
    __description__,
    __pkg_name__,
    __version__,
)

__all__ = (
    "__authors__",
    "__build_profile__",
    "__build_timestamp__",
    "__description__",
    "__pkg_name__",
    "__pkgroot__",
    "__title__",
    "__version__",
    "__version__",
)
__title__ = "utiles"
__pkgroot__ = __file__.replace("__about__.py", "").rstrip("/\\")
