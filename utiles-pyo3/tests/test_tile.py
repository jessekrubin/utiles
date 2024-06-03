import utiles as ut


def test_tile_center():
    center = ut.Tile(0, 0, 0).center()
    assert center == (0, 0)
