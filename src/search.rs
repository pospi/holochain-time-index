use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Timelike, Utc};
use hdk::{hash_path::path::Component, prelude::*};

use crate::entries::IndexType;
use crate::errors::{IndexError, IndexResult};
use crate::INDEX_DEPTH;

pub(crate) fn get_naivedatetime(
    from: &DateTime<Utc>,
    until: &DateTime<Utc>,
    index_type: &IndexType,
) -> Option<(NaiveDateTime, NaiveDateTime)> {
    match index_type {
        IndexType::Year => Some((
            NaiveDate::from_ymd(from.year(), 1, 1).and_hms(1, 1, 1),
            NaiveDate::from_ymd(until.year(), 1, 1).and_hms(1, 1, 1),
        )),
        IndexType::Month => Some((
            NaiveDate::from_ymd(from.year(), from.month(), 1).and_hms(1, 1, 1),
            NaiveDate::from_ymd(until.year(), until.month(), 1).and_hms(1, 1, 1),
        )),
        IndexType::Day => Some((
            NaiveDate::from_ymd(from.year(), from.month(), from.day()).and_hms(1, 1, 1),
            NaiveDate::from_ymd(until.year(), until.month(), until.day()).and_hms(1, 1, 1),
        )),
        IndexType::Hour => {
            if INDEX_DEPTH.contains(&index_type) {
                Some((
                    NaiveDate::from_ymd(from.year(), from.month(), from.day()).and_hms(
                        from.hour(),
                        1,
                        1,
                    ),
                    NaiveDate::from_ymd(until.year(), until.month(), until.day()).and_hms(
                        until.hour(),
                        1,
                        1,
                    ),
                ))
            } else {
                None
            }
        }
        IndexType::Minute => {
            if INDEX_DEPTH.contains(&index_type) {
                Some((
                    NaiveDate::from_ymd(from.year(), from.month(), from.day()).and_hms(
                        from.hour(),
                        from.minute(),
                        1,
                    ),
                    NaiveDate::from_ymd(until.year(), until.month(), until.day()).and_hms(
                        until.hour(),
                        until.minute(),
                        1,
                    ),
                ))
            } else {
                None
            }
        }
        IndexType::Second => {
            if INDEX_DEPTH.contains(&index_type) {
                Some((
                    NaiveDate::from_ymd(from.year(), from.month(), from.day()).and_hms(
                        from.hour(),
                        from.minute(),
                        from.second(),
                    ),
                    NaiveDate::from_ymd(until.year(), until.month(), until.day()).and_hms(
                        until.hour(),
                        until.minute(),
                        until.second(),
                    ),
                ))
            } else {
                None
            }
        }
    }
}

/// Tries to find the newest time period one level down from current path position
/// Returns path passed in params if maximum depth has been reached
pub(crate) fn find_newest_time_path<
    T: TryFrom<SerializedBytes, Error = SerializedBytesError> + Into<u32>,
>(
    path: Path,
    time_index: IndexType,
) -> IndexResult<Path> {
    match time_index {
        IndexType::Year => (),
        IndexType::Month => (),
        IndexType::Day => (),
        IndexType::Hour => {
            if INDEX_DEPTH.contains(&time_index) {
                ()
            } else {
                return Ok(path);
            }
        }
        IndexType::Minute => {
            if INDEX_DEPTH.contains(&time_index) {
                ()
            } else {
                return Ok(path);
            }
        }
        IndexType::Second => {
            if INDEX_DEPTH.contains(&time_index) {
                ()
            } else {
                return Ok(path);
            }
        }
    };
    //debug!("Finding links on IndexType: {:#?}\n\n", time_index);

    //Pretty sure this filter and sort logic can be faster; first rough pass to get basic pieces in place
    let mut links = path.children_paths()?;
    if links.len() == 0 {
        return Err(IndexError::InternalError(
            "Could not find any time paths for path",
        ));
    };
    links.sort_by(|a, b| {
        let a_val: Vec<Component> = a.to_owned().into();
        let b_val: Vec<Component> = b.to_owned().into();
        let a_u32: u32 = T::try_from(SerializedBytes::from(UnsafeBytes::from(
            a_val[1].as_ref().to_owned(),
        )))
        .unwrap()
        .into();
        let b_u32: u32 = T::try_from(SerializedBytes::from(UnsafeBytes::from(
            b_val[1].as_ref().to_owned(),
        )))
        .unwrap()
        .into();
        a_u32.partial_cmp(&b_u32).unwrap()
    });
    let latest = links.pop().unwrap();
    Ok(latest)
}
