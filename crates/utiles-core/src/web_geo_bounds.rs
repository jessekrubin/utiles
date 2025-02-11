use crate::wrap_lon;

/// A bounding box in (west, south, east, north) format.
/// Note that `west > east` implies crossing the antimeridian.
pub(crate) type TBounds = (f64, f64, f64, f64);

/// Computes a "union" bounds/bbox that contains/encloses input bounds/bboxes.
/// The resulting bbox can cross the antimeridian (i.e. `west > east`).
///
/// Special cases:
/// - If there is only **one** bbox, we return it exactly (even if it crosses the antimeridian).
/// - If there are no bboxes, we return None
///
/// Examples:
///
/// Multiple bboxes that some of which cross the antimeridian:
/// ```
/// use utiles_core::web_geo_bounds_union;
/// let bboxes = vec![
///     (170.0, -10.0, -170.0, 10.0), // crosses AM
///     (-160.0, -20.0, -100.0, 5.0),
///     (120.0, -15.0, 160.0, 15.0),
/// ];
/// let bbox = web_geo_bounds_union(&bboxes).unwrap();
/// assert_eq!(bbox, (120.0, -20.0, -100.0, 15.0));
/// ```
///
/// Single bbox that crosses the antimeridian:
/// ```
/// use utiles_core::web_geo_bounds_union;
/// let bboxes = vec![(170.0, -10.0, -170.0, 5.0)];
/// let bbox = web_geo_bounds_union(&bboxes).unwrap();
/// assert_eq!(bbox, (170.0, -10.0, -170.0, 5.0));
/// ```
#[must_use]
pub fn web_geo_bounds_union(bboxes: &[TBounds]) -> Option<TBounds> {
    if bboxes.is_empty() {
        return None;
    }
    // collect:
    // 1) the min/max lat as that is going to be our min/max lat...
    // AND
    // 2) convert each bbox into one or two longitude ranges
    let (south, north, mut ranges) = collect_minmax_lat_and_lng_ranges(bboxes);
    // merge the ranges that overlap or are adjacent into contiguous ranges
    let merged = merge_lng_ranges(&mut ranges);
    // return if only one range
    if merged.len() == 1 {
        let (final_west, final_east) = merged[0];
        return Some((final_west, south, final_east, north));
    }

    // find the largest void/gap/hole in the merged ranges, as that is the
    // arc that is opposite of our desired bounds
    let (gap_start, gap_end) = largest_lng_range_hole(&merged);
    let west = wrap_lon(gap_end);
    let east = wrap_lon(gap_start);
    Some((west, south, east, north))
}

/// Gathers min/max latitude, and converts each bbox into one or two longitude intervals.
/// Returns (`min_lat`, `max_lat`, Vec<(start, end)>).
fn collect_minmax_lat_and_lng_ranges(
    bboxes: &[TBounds],
) -> (f64, f64, Vec<(f64, f64)>) {
    bboxes.iter().fold(
        (f64::INFINITY, f64::NEG_INFINITY, Vec::new()),
        |(min_lat, max_lat, mut ranges), &(west, south, east, north)| {
            // Update latitude boundaries
            let new_min_lat = min_lat.min(south);
            let new_max_lat = max_lat.max(north);

            // Convert to intervals (handle crossing the antimeridian)
            if west > east {
                ranges.push((west, 180.0));
                ranges.push((-180.0, east));
            } else {
                ranges.push((west, east));
            }

            (new_min_lat, new_max_lat, ranges)
        },
    )
}

fn merge_lng_ranges(ranges: &mut [(f64, f64)]) -> Vec<(f64, f64)> {
    if ranges.is_empty() {
        return Vec::new();
    }
    if ranges.len() == 1 {
        return vec![ranges[0]];
    }

    // sort by start... bc we gotta if there are more than one...
    ranges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // init and add first one...
    let mut merged = Vec::with_capacity(ranges.len());
    merged.push(ranges[0]);

    // fold fold fold
    ranges[1..].iter().fold(merged, |mut acc, &(start, end)| {
        if let Some((_prev_start, prev_end)) = acc.last_mut() {
            // diff between
            let abs_diff = (start - *prev_end).abs();
            // 0.0001 is close enough
            if abs_diff <= 0.0001 {
                *prev_end = prev_end.max(end);
            } else {
                acc.push((start, end));
            }
        }

        // gotta return the acc which I forgot to do and was tearing my hair out
        // for a while... over this... super... dumb... mistake...
        acc
    })
}

/// Finds the largest gap on the circular [-180..180] domain
/// among the merged intervals, returning (`gap_start`, `gap_end`).
///
/// The intervals here are non-overlapping and sorted.
/// The "largest gap" is the segment NOT covered by intervals.
fn largest_lng_range_hole(merged: &[(f64, f64)]) -> (f64, f64) {
    if merged.is_empty() {
        return (0.0, 0.0);
    }
    let (largest_gap, gap_start, gap_end) = merged.windows(2).fold(
        (-1.0, 0.0, 0.0), // (acc_gap_size, acc_gap_start, acc_gap_end) initial
        |(acc_gap, acc_start, acc_end), w| {
            let curr_end = w[0].1; // end of the first interval
            let next_start = w[1].0; // start of the next interval
            let gap_size = next_start - curr_end;
            if gap_size > acc_gap {
                (gap_size, curr_end, next_start)
            } else {
                (acc_gap, acc_start, acc_end)
            }
        },
    );
    let last_end = merged.last().unwrap_or(&(0.0, 0.0)).1;
    let first_start = merged.first().unwrap_or(&(0.0, 0.0)).0;
    let wrap_size = (first_start + 360.0) - last_end;
    // if the wrap-around is bigger than the largest gap, then
    // we return that instead
    if wrap_size > largest_gap {
        (last_end, first_start + 360.0)
    } else {
        (gap_start, gap_end)
    }
}

/// Tests for the `web_geo_bounds_union` function.
///
/// I wrote this in python to start with and verified these w/ the python
/// version which I dumped into geojson.io to verify visually with my eyeballs
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    #[test]
    fn test_single_normal() {
        let input = vec![(100.0, -5.0, 120.0, 10.0)];
        let bbox = web_geo_bounds_union(&input).unwrap();
        assert_eq!(bbox, (100.0, -5.0, 120.0, 10.0));
    }

    #[test]
    fn test_single_crossing_am() {
        // The single box crosses the antimeridian (west > east).
        // We should return exactly what was passed in.
        let input = vec![(170.0, -10.0, -170.0, 5.0)];
        let bbox = web_geo_bounds_union(&input).unwrap();
        assert_eq!(bbox, (170.0, -10.0, -170.0, 5.0));
    }

    #[test]
    fn test_multiple_merged() {
        let input = vec![
            (170.0, -10.0, -170.0, 10.0), // crosses AM
            (-160.0, -20.0, -100.0, 5.0),
            (120.0, -15.0, 160.0, 15.0),
        ];
        let bbox = web_geo_bounds_union(&input).unwrap();
        let expected: TBounds = (120.0, -20.0, -100.0, 15.0);
        assert_eq!(bbox, expected);
    }

    #[test]
    fn test_two_not_crossing_antimeridian() {
        let input = vec![(100.0, -5.0, 120.0, 10.0), (110.0, -10.0, 130.0, 5.0)];
        let bbox = web_geo_bounds_union(&input).unwrap();
        assert_eq!(bbox, (100.0, -10.0, 130.0, 10.0));
    }

    #[test]
    fn test_no_bboxes() {
        let input = vec![];
        let bbox = web_geo_bounds_union(&input);
        assert_eq!(bbox, None);
    }
}
