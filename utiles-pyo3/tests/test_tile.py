from __future__ import annotations

import pickle

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


def test_pickling() -> None:
    pickled = pickle.dumps(ut.Tile(0, 0, 0))
    loaded = pickle.loads(pickled)
    assert loaded == ut.Tile(0, 0, 0)


def test_tile_children_zorder() -> None:
    children = ut.Tile(0, 0, 0).children(zorder=True)
    children_1 = [
        ut.Tile(x=0, y=0, z=1),
        ut.Tile(x=1, y=0, z=1),
        ut.Tile(x=0, y=1, z=1),
        ut.Tile(x=1, y=1, z=1),
    ]
    assert children == children_1

    children_2 = ut.Tile(0, 0, 0).children(2, zorder=True)

    children_1 = [
        item
        for sublist in [c.children(zorder=True) for c in children_1]
        for item in sublist
    ]

    assert children_2 == children_1
