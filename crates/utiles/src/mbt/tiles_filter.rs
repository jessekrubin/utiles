use utiles_core::{tile_ranges, BBox, ZoomOrZooms, ZoomSet};

use crate::errors::UtilesResult;

#[derive(Debug, Clone)]
pub struct TilesFilter {
    pub bboxes: Option<Vec<BBox>>,
    pub zooms: Option<Vec<u8>>,
}

impl TilesFilter {
    #[must_use]
    pub fn new(bboxes: Option<Vec<BBox>>, zooms: Option<Vec<u8>>) -> Self {
        Self { bboxes, zooms }
    }

    pub fn mbtiles_sql_where(&self, prefix: Option<&str>) -> UtilesResult<String> {
        self.where_clause(prefix)
    }

    pub fn where_clause(&self, prefix: Option<&str>) -> UtilesResult<String> {
        let pred = match (&self.bboxes, &self.zooms) {
            (Some(bbox), Some(zooms)) => {
                let zboxes = bbox
                    .iter()
                    .flat_map(|b| {
                        tile_ranges(b.tuple(), ZoomOrZooms::Zooms(zooms.clone()))
                    })
                    .collect::<Vec<_>>();
                let pred = zboxes
                    .iter()
                    .map(|a| a.mbtiles_sql_where(prefix))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                format!("({pred})")
            }
            (Some(bbox), None) => {
                let zboxes = bbox
                    .iter()
                    .flat_map(|b| {
                        tile_ranges(
                            b.tuple(),
                            ZoomOrZooms::Zooms(ZoomSet::all().into()),
                        )
                    })
                    .collect::<Vec<_>>();
                let pred = zboxes
                    .iter()
                    .map(|a| a.mbtiles_sql_where(prefix))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                format!("({pred})")
            }
            (None, Some(zooms)) => {
                format!(
                    "zoom_level IN ({zooms})",
                    zooms = zooms
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
            (None, None) => String::new(),
        };
        // attach 'WHERE'
        if pred.is_empty() {
            Ok(pred)
        } else {
            Ok(format!("WHERE {pred}"))
        }
    }
}
