import utiles as ut


def test_tile_fmts_simple():
    t = (1, 2, 3)
    tfmt = ut.TileFmts("{z}/{x}/{y}")
    assert tfmt.format(t) == "3/1/2"
    assert str(tfmt) == "TileFmts({z}/{x}/{y})"

    tfmt_yup_png = ut.TileFmts("{z}/{x}/{-y}.png")
    assert tfmt_yup_png.format(t) == "3/1/5.png"
    assert str(tfmt_yup_png) == "TileFmts({z}/{x}/{-y}.png)"


tile_formatter = ut.TileFmts("https://tile.openstreetmap.org/{z}/{x}/{y}.png")


def _osm_tile_url_fmtstr(t: ut.Tile) -> str:
    return tile_formatter.format(t)


def _osm_tile_url_fstring(t: ut.Tile) -> str:
    return f"https://tile.openstreetmap.org/{t.z}/{t.x}/{t.y}.png"


def test_equiv():
    assert _osm_tile_url_fmtstr(ut.Tile(1, 2, 3)) == _osm_tile_url_fstring(
        ut.Tile(1, 2, 3)
    )
