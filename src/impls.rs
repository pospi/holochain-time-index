//use chrono::{Duration, DurationRound};

use std::convert::{TryFrom, TryInto};

use chrono::{NaiveDate, NaiveDateTime};
use hdk3::{
    hash_path::path::{Component, Path},
    prelude::{ExternResult, SerializedBytes, UnsafeBytes, WasmError},
};

use crate::entries::{
    DayIndex, HourIndex, Index, IndexIndex, MinuteIndex, MonthIndex, SecondIndex, WrappedPath,
    YearIndex,
};

impl IndexIndex {
    pub fn get_sb(self) -> ExternResult<SerializedBytes> {
        Ok(self.try_into()?)
    }
}

impl TryFrom<Path> for Index {
    type Error = WasmError;

    fn try_from(data: Path) -> ExternResult<Index> {
        let path_comps: Vec<Component> = data.into();
        let time_index = path_comps
            .last()
            .ok_or(WasmError::Zome(String::from(
                "Cannot get Index from empty path",
            )))?
            .to_owned();
        let time_index: Vec<u8> = time_index.into();
        let time_index = Index::try_from(SerializedBytes::from(UnsafeBytes::from(time_index)))?;
        Ok(time_index)
    }
}

impl TryInto<NaiveDateTime> for WrappedPath {
    type Error = WasmError;

    fn try_into(self) -> Result<NaiveDateTime, Self::Error> {
        let data = self.0;
        let path_comps: Vec<Component> = data.into();
        Ok(NaiveDate::from_ymd(
            path_comps.get(1).ok_or(WasmError::Zome(String::from(
                "Expected at least two elements to convert to DateTime",
            )))?,
            path_comps.get(2).unwrap_or(1),
            path_comps.get(3).unwrap_or(1),
        )
        .and_hms(
            path_comps.get(4).unwrap_or(1),
            path_comps.get(5).unwrap_or(1),
            path_comps.get(6).unwrap_or(1),
        ))
    }
}

impl From<u32> for YearIndex {
    fn from(data: u32) -> Self {
        YearIndex(data)
    }
}

impl Into<u32> for YearIndex {
    fn into(self) -> u32 {
        self.0
    }
}

impl From<u32> for MonthIndex {
    fn from(data: u32) -> Self {
        MonthIndex(data)
    }
}

impl Into<u32> for MonthIndex {
    fn into(self) -> u32 {
        self.0
    }
}

impl From<u32> for DayIndex {
    fn from(data: u32) -> Self {
        DayIndex(data)
    }
}

impl Into<u32> for DayIndex {
    fn into(self) -> u32 {
        self.0
    }
}

impl From<u32> for HourIndex {
    fn from(data: u32) -> Self {
        HourIndex(data)
    }
}

impl Into<u32> for HourIndex {
    fn into(self) -> u32 {
        self.0
    }
}

impl From<u32> for MinuteIndex {
    fn from(data: u32) -> Self {
        MinuteIndex(data)
    }
}

impl Into<u32> for MinuteIndex {
    fn into(self) -> u32 {
        self.0
    }
}

impl From<u32> for SecondIndex {
    fn from(data: u32) -> Self {
        SecondIndex(data)
    }
}

impl Into<u32> for SecondIndex {
    fn into(self) -> u32 {
        self.0
    }
}
