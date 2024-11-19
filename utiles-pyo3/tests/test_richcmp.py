import utiles as ut


def test_equality() -> None:
    assert ut.Tile(1, 2, 3) == (1, 2, 3)
    assert ut.Tile(1, 2, 3) == ut.Tile(1, 2, 3)
    assert (1, 2, 3) == ut.Tile(1, 2, 3)


def test_equality_invalid_zoom() -> None:
    utile = ut.Tile(1, 2, 3)
    tuple_ = (1, 2, 2234234)
    assert tuple_ != utile
    assert utile != tuple_

    assert (1, 2, 2234234) != ut.Tile(1, 2, 3)
