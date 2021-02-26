use chrono::{DateTime, NaiveDateTime, Utc};
use hdk3::{
    hash_path::{anchor::Anchor, path::Component},
    prelude::*,
};

use crate::utils::{get_time_path, add_time_index_to_path};
use crate::{
    DayIndex, HourIndex, MinuteIndex, MonthIndex, SecondIndex, TimeChunk, YearIndex,
    MAX_CHUNK_INTERVAL, TimeIndex
};

impl TimeChunk {
    /// Create a new chunk & link to time index
    pub fn create_chunk(&self, is_genesis: bool) -> HdkResult<()> {
        //These validations are to help zome callers; but should also be present in validation rules
        if self.from > sys_time()? {
            return Err(HdkError::Wasm(WasmError::Zome(String::from(
                "Time chunk cannot start in the future",
            ))));
        };
        if self.until - self.from != *MAX_CHUNK_INTERVAL {
            return Err(HdkError::Wasm(WasmError::Zome(String::from(
                "Time chunk should use period equal to max interval set by DNA",
            ))));
        };
        if !is_genesis {
            //Genesis should actually be embedded into DHT via properties; this saves lookup on each insert/validation
            let genesis = get_genesis_chunk()?.ok_or(HdkError::Wasm(WasmError::Zome(
                String::from("Time chunk cannot be created until gensis chunk has been created"),
            )))?;
            if (self.from - genesis.from).as_millis() % MAX_CHUNK_INTERVAL.as_millis() != 0 {
                return Err(HdkError::Wasm(WasmError::Zome(String::from(
                    "Time chunk does not follow chunk interval ordering since genesis",
                ))));
            };
        };

        let time_path = get_time_path(self.from)?;

        //Create the TimeChunk entry
        create_entry(self)?;

        //Link TimeChunk entry to time tree
        let time_path = Path::from(time_path);
        time_path.ensure()?;
        create_link(time_path.hash()?, self.hash()?, LinkTag::new("chunk"))?;

        //Genesis should likely be created on DNA init and passed in via properties to avoid DHT lookup for each insert
        if is_genesis {
            let genesis_anchor = Anchor {
                anchor_type: String::from("genesis"),
                anchor_text: None,
            };
            create_entry(&genesis_anchor)?;
            create_link(
                hash_entry(&genesis_anchor)?,
                self.hash()?,
                LinkTag::new("genesis"),
            )?;
        };
        Ok(())
    }

    /// Return the hash of the entry
    pub fn hash(&self) -> HdkResult<EntryHash> {
        hash_entry(self)
    }

    /// Reads current chunk and moves back N step intervals and tries to get that chunk
    pub fn get_previous_chunk(&self, back_steps: u32) -> HdkResult<Option<TimeChunk>> {
        let last_chunk = TimeChunk {
            from: self.from - (*MAX_CHUNK_INTERVAL * back_steps),
            until: self.until - (*MAX_CHUNK_INTERVAL * back_steps),
        };
        match get(last_chunk.hash()?, GetOptions::content())? {
            Some(chunk) => Ok(Some(chunk.entry().to_app_option()?.ok_or(
                HdkError::Wasm(WasmError::Zome(String::from(
                    "Could not deserialize link target into TimeChunk",
                ))),
            )?)),
            None => Ok(None),
        }
    }

    /// Get latest chunk using sys_time as source for latest time
    pub fn get_latest_chunk() -> HdkResult<Option<TimeChunk>> {
        //Running with the asumption here that sys_time is always UTC
        let now = sys_time()?;
        let now = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(now.as_secs_f64() as i64, now.subsec_nanos()),
            Utc,
        );
        //Create current time path
        let mut time_path = vec![];
        add_time_index_to_path::<YearIndex>(&mut time_path, &now, TimeIndex::Year)?;
        add_time_index_to_path::<MonthIndex>(&mut time_path, &now, TimeIndex::Month)?;
        add_time_index_to_path::<DayIndex>(&mut time_path, &now, TimeIndex::Day)?;
        add_time_index_to_path::<HourIndex>(&mut time_path, &now, TimeIndex::Hour)?;
        add_time_index_to_path::<MinuteIndex>(&mut time_path, &now, TimeIndex::Minute)?;
        add_time_index_to_path::<SecondIndex>(&mut time_path, &now, TimeIndex::Second)?;
        let time_path = Path::from(time_path);

        let chunks = get_links(time_path.hash()?, Some(LinkTag::new("chunk")))?;
        let mut latest_chunk = chunks.into_inner();
        latest_chunk.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());

        match latest_chunk.pop() {
            Some(link) => match get(link.target, GetOptions::content())? {
                Some(chunk) => Ok(Some(chunk.entry().to_app_option()?.ok_or(
                    HdkError::Wasm(WasmError::Zome(String::from(
                        "Could not deserialize link target into TimeChunk",
                    ))),
                )?)),
                None => Ok(None),
            },
            None => Ok(None)
        }
    }

    // /// Tries to get chunk with current timestamp; if it cannot find a chunk then it will keep going back until it finds one
    // /// Max depth decides how far back we will look to get a chunk. Note max depth travels across time types.
    // pub fn get_latest_chunk_recursive(max_depth: u32) -> HdkResult<TimeChunk> {
    //     //TODO
    //     //First try to get chunk on current time; if None then
    //     //Move back back one time index; where the time index to move back by is the smallest value we index by 
    //     //as denoted by TIME_INDEX_DEPTH
    //     //This has to happen until IndexType has reached lowest value; at which point it will decrement the index value above
    //     //the current in tree heirachy
    // }

    /// Get all chunks that exist for some time period between from -> until
    pub fn get_chunks_for_time_span(
        from: DateTime<Utc>,
        until: DateTime<Utc>,
    ) -> HdkResult<Vec<EntryHash>> {
        //Check that timeframe specified is greater than the TIME_INDEX_DEPTH.
        //If it is lower then no results will ever be returned
        //Next is to deduce how tree should be traversed and what time index level/path(s)
        //to be used to find chunks
        Ok(vec![])
    }

    pub fn add_link(&self, target: EntryHash) -> HdkResult<()> {
        //TODO
        //Read how many links an agent already has on a given chunk
        //If under DIRECT_CHUNK_LINK_LIMIT then make direct link
        //otherwise create linked list starting from latest link or latest link in chain of links
        Ok(())
    }

    pub fn get_links(&self, limit: u32) -> HdkResult<Vec<EntryHash>>{
        //TODO
        //Read for direct links on chunk as well as traverse into any linked list on a chunk to find
        //any other linked addresses
        Ok(vec![])
    }

    pub fn validate_chunk(&self) -> HdkResult<()> {
        //TODO: incorrect error type being used here
        if self.from > sys_time()? {
            return Err(HdkError::Wasm(WasmError::Zome(String::from(
                "Time chunk cannot start in the future",
            ))));
        };
        if self.until - self.from != *MAX_CHUNK_INTERVAL {
            return Err(HdkError::Wasm(WasmError::Zome(String::from(
                "Time chunk should use period equal to max interval set by DNA",
            ))));
        };
        if self.until - self.from != *MAX_CHUNK_INTERVAL {
            return Err(HdkError::Wasm(WasmError::Zome(String::from(
                "Time chunk should use period equal to max interval set by DNA",
            ))));
        };
        //Genesis should actually be embedded into DHT via properties; this saves lookup on each insert/validation
        let genesis = get_genesis_chunk()?.ok_or(HdkError::Wasm(WasmError::Zome(
            String::from("Time chunk cannot be created until gensis chunk has been created"),
        )))?;
        if (self.from - genesis.from).as_millis() % MAX_CHUNK_INTERVAL.as_millis() != 0 {
            return Err(HdkError::Wasm(WasmError::Zome(String::from(
                "Time chunk does not follow chunk interval ordering since genesis",
            ))));
        };
        Ok(())
    }

    // pub fn validate_chunk_link(&self, link: LinkData) -> HdkResult<()> {
    //     //Interesting interplay developing here
    //     //The complexity to make one link increases with number of links on that chunk
    //     //Thus you could say its worth making chunks as small as possible
    //     //But then you may get added retrieval complexity for a given timeperiod
    //     //I.e having to ask for links on 100 individual second chunks vs two 50 second chunks
    //     //You could probably algorithmically deduce the ideal value for retrival vs commit intensity
    //     if get_links(self.hash(), None).filter(|commited_link| commited_link.author == link.author).count() -1 > DIRECT_CHUNK_LIMIT {
    //         return Err(())
    //     } else {
    //         return Ok(())
    //     }
    // }
}

/// Tries to find the first chunk committed; i.e the "genesis chunk"
pub fn get_genesis_chunk() -> HdkResult<Option<TimeChunk>> {
    let genesis_anchor = Anchor {
        anchor_type: String::from("genesis"),
        anchor_text: None,
    };
    let genesis_hash = hash_entry(&genesis_anchor)?;
    let links = get_links(genesis_hash, Some(LinkTag::new("genesis")))?;
    let time_chunk = match links.into_inner().first() {
        Some(link) => {
            let element = get(link.target.to_owned(), GetOptions::default())?;
            match element {
                Some(element) => Some(element.entry().to_app_option()?.ok_or(HdkError::Wasm(
                    WasmError::Zome(String::from(
                        "Could not deserialize link target into TimeChunk",
                    )),
                ))?),
                None => None,
            }
        }
        None => None,
    };
    Ok(time_chunk)
}

// /// Will take current time and try to find a chunk that fits; if no chunk is found then it will create a chunk that fits
// pub fn get_valid_chunk() -> HdkResult<TimeChunk> {
//     //TODO:
//     //determine how many hops from genesis until value until we would get a chunk where the 
//     //current time would fit inside the from & until of chunk. Try to get this chunk
//     //if it does not exist then create it and return other wise just return retrieved chunk
//     Ok(TimeChunk)
// }