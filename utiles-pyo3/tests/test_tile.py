from __future__ import annotations

import utiles as ut


def test_tile_center() -> None:
    center = ut.Tile(0, 0, 0).center()
    assert center == (0, 0)


def test_tile_children() -> None:
    children = ut.Tile(0, 0, 0).children()
    children_1 = [
        ut.Tile(x=0, y=0, z=1),
        ut.Tile(x=1, y=0, z=1),
        ut.Tile(x=1, y=1, z=1),
        ut.Tile(x=0, y=1, z=1),
    ]
    assert children == children_1

    children_2 = ut.Tile(0, 0, 0).children(2)

    children_1 = [
        item for sublist in [c.children() for c in children_1] for item in sublist
    ]

    assert children_2 == children_1
