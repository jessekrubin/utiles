use std::path::PathBuf;

use serde::Serialize;

use utiles_core::zoom::{ZoomOrZooms, ZoomSet};
use utiles_core::{tile_ranges, BBox};

use crate::errors::{UtilesCopyError, UtilesResult};

#[derive(Debug, Clone, Serialize, Default)]
pub struct CopyConfig {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub zset: Option<ZoomSet>,
    pub zooms: Option<Vec<u8>>,
    pub bboxes: Option<Vec<BBox>>,
    pub verbose: bool,
    pub dryrun: bool,
    pub force: bool,
    pub jobs: Option<u8>,
}

impl CopyConfig {
    pub fn mbtiles_sql_where(
        &self,
        // zoom_levels: Option<Vec<u8>>,
    ) -> UtilesResult<String> {
        let pred = match (&self.bboxes, &self.zooms) {
            (Some(bbox), Some(zooms)) => {
                // let zooms =  self.zooms.unwrap_or(ZoomSet::all().into());
                let zboxes = bbox
                    .iter()
                    .flat_map(|b| {
                        tile_ranges(b.tuple(), ZoomOrZooms::Zooms(zooms.clone()))
                    })
                    .collect::<Vec<_>>();
                let pred = zboxes
                    .iter()
                    .map(utiles_core::tile_zbox::TileZBoxes::mbtiles_sql_where)
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
                    .map(utiles_core::tile_zbox::TileZBoxes::mbtiles_sql_where)
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
    pub fn check_src_dst_same(&self) -> UtilesResult<()> {
        if self.src == self.dst {
            Err(UtilesCopyError::SrcDstSame(format!(
                "src: {:?}, dst: {:?}",
                self.src, self.dst
            ))
            .into())
        } else {
            Ok(())
        }
    }

    pub fn check_src_exists(&self) -> UtilesResult<()> {
        if self.src.exists() {
            Ok(())
        } else {
            Err(UtilesCopyError::SrcNotExists(format!("src: {:?}", self.src)).into())
        }
    }

    pub fn check(&self) -> UtilesResult<()> {
        self.check_src_exists()?;
        self.check_src_dst_same()?;
        Ok(())
    }

    pub fn njobs(&self) -> u8 {
        if let Some(j) = self.jobs {
            j
        } else {
            let ncpus = num_cpus::get();
            // if less than 4 cpus then use 1 job otherwise just default to 4 to
            // not throttle errything
            if ncpus < 4 {
                1
            } else {
                4
            }
        }
    }
}

//
// impl crate::cli::commands::copy::CopyConfigV1 {
//     pub fn new(
//         src: crate::cli::commands::copy::Source,
//         dst: crate::cli::commands::copy::Destination,
//         zooms: Option<Vec<u8>>,
//         bbox: Option<BBox>,
//     ) -> Self {
//         Self {
//             src,
//             dst,
//             zooms,
//             bbox,
//         }
//     }
//
//     // pub fn sql_where_for_zoom(&self, zoom: u8) -> String {
//     //     let pred = match &self.bbox {
//     //         Some(bbox) => {
//     //             let trange = tile_ranges(bbox.tuple(), vec![zoom].into());
//     //             trange.sql_where(Some(true))
//     //         }
//     //         None => {
//     //             format!("zoom_level = {zoom}")
//     //         }
//     //     };
//     //     // attach 'WHERE'
//     //     if pred.is_empty() {
//     //         pred
//     //     } else {
//     //         format!("WHERE {pred}")
//     //     }
//     // }
//
//     pub fn mbtiles_sql_where(
//         &self,
//         zoom_levels: Option<Vec<u8>>,
//     ) -> UtilesResult<String> {
//         let pred = match (&self.bbox, &self.zooms) {
//             (Some(bbox), Some(zooms)) => {
//                 let trange = tile_ranges(
//                     bbox.tuple(),
//                     zoom_levels.unwrap_or(zooms.clone()).into(),
//                 )?;
//                 trange.mbtiles_sql_where()
//             }
//             (Some(bbox), None) => {
//                 let trange = tile_ranges(
//                     bbox.tuple(),
//                     zoom_levels
//                         .unwrap_or((0..28).map(|z| z as u8).collect::<Vec<u8>>())
//                         .into(),
//                 )?;
//                 trange.mbtiles_sql_where()
//             }
//             (None, Some(zooms)) => {
//                 format!(
//                     "zoom_level IN ({zooms})",
//                     zooms = zooms
//                         .iter()
//                         .map(std::string::ToString::to_string)
//                         .collect::<Vec<String>>()
//                         .join(",")
//                 )
//             }
//             (None, None) => String::new(),
//         };
//         // attach 'WHERE'
//         if pred.is_empty() {
//             Ok(pred)
//         } else {
//             Ok(format!("WHERE {pred}"))
//         }
//     }
// }
