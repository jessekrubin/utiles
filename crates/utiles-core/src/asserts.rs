use crate::{TileLike, UtilesCoreError, UtilesCoreResult};

pub(crate) fn assert_all_same_zoom<I>(tiles: I) -> UtilesCoreResult<u8>
where
    I: IntoIterator,   // slice, Vec, HashSet, iterator adapters … all work
    I::Item: TileLike, // every yielded value (owned **or** borrowed) is a tile
{
    let mut zoom: Option<u8> = None;

    for tile in tiles {
        let z = tile.zoom();
        match zoom {
            Some(prev) if prev != z => {
                return Err(UtilesCoreError::AdHoc(
                    "Tiles have different zoom levels".to_string(),
                ));
            }
            _ => zoom = Some(z),
        }
    }
    zoom.ok_or(UtilesCoreError::AdHoc("No tiles provided".to_string()))
}
