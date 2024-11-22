from __future__ import annotations

import utiles as ut


def test_lnglat_false() -> None:
    center = ut.Tile(0, 0, 0).center()
    assert center != (0, 0, 0)
