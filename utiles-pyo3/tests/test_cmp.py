import utiles as ut


def test_lnglat_false():
    center = ut.Tile(0, 0, 0).center()
    assert center != (0, 0, 0)
