# HC-Time-Chunking

## Purpose

This DHT aims to be one solution (of many) to the DHT hostpotting problem that can occur in holochain DHT's when many links are made from one entry.
This hotspotting occurs as the original author (and their surrounding hash neighbourhood?) of an entry is responsible for storing and resolving all links from the given authored entry. As a result if a given entry becomes very popular then it can be left up to one or a few nodes to handle all traffic flowing through this part of the DHT.

## Function

The main component that allows the mitigation of DHT hotspots are: 
1) time delimited chunks.
2) agent centric validation that occurs on each chunk.

### Time Delimited Chunks

The way the chunks are implemented creates a sort of "emergent" linked list behaviour. Emergent because they are not actually linked together; but instead all use the same `MAX_CHUNK_LIMIT` for the timespace they can serve. Thus any given chunk are N chunks away from the genesis chunk and fit into some kind of ordered/connected list. This is useful as it allows for us to determine if a given chunks timespace is allowed in validation. Implementing a traditional linked list over chunks is hard due to the difficulty of gathering consensus about what chunk was the last chunk for some given chunk and which one is the next; each agent might have a different view or perspective over the DHT's data and thus might see different past and future chunks. Enforcing that all chunks serve a `MAX_CHUNK_LIMIT` its easy for any agent to derive what chunks may or may not exist.

### Agent Link Validation

For any given chunk an **agent** cannot make more than `DIRECT_CHUNK_LINK_LIMIT` direct links on a given chunk. Once this limit has been met, subsequent links must be linked together in a linked list shape. 
Here the target entry of the last direct link they created is the source entry of the linked list. An agent can make links like this until their total links reaches the `ENFORCE_SPAM_LIMIT` limit at which point no further links are allowed on this chunk. 

The first limit is a measure to protect DHT hotspots in a busy DHT with a high `MAX_CHUNK_INTERVAL` & the second limit is supposed to block obvious spam.

### DNA Lifecycle

This DNA's variables mentioned above are expected to be static. That means its expected that the: `DIRECT_CHUNK_LINK_LIMIT`, `ENFORCE_SPAM_LIMIT` & `MAX_CHUNK_INTERVAL` will stay the same throughout the lifetime of the DHT. This is done to make validation possible in situations where DHT sharding could occur. 
If limits are able to change; we have no way to reliably know if an agent is operating on old limits by consequence of being out of touch with latest DHT state or if the agent is malicious and pretending they do not see the new limits. You can see this being an especially big problem when you have two areas of the DHT "merging" and the "outdated" area of the DHT having all of its links in-validated by the agents in the more current of the DHT space.

Currently if we wish to update limits we will create a new DNA/DHT and link to the new one from the current.

If you can guarantee that fragmentation of the DHT will not happen then its possible to implement limit updates. If this is something you wish to do its recommended that you enforce new limits at some given chunk in the future rather than instantly. This allows you to (hopefully) give enough time for other DHT agents to receive new limit information before its enforced.   

### Exposed Functions

This DNA exposes a few helper functions to make operating with chunked data easy. Ones of note are: 
`get_current_chunk()`, `get_latest_chunk()`, `get_chunks_for_time_span()`, `add_link()` & `get_links()`:

- `get_current_chunk()` will take the current time as denoted by `sys_time()` and return either null or a chunk that can be used to served entries for the current time.<br>
- `get_latest_chunk()` will search though the DNA's time "index" and find the last commited chunk and return it.<br>
- `get_chunks_for_time_span()` will return all chunks that served in a given time span.<br>
- `add_link()` will create a link on a chunk. This will happen either directly or by the linked list fashion as explained above.<br>
- `get_links()` will get links from the chunk, recursing down any linked lists to ensure that all links are returned for a given chunk.<br>

### hApp Usage

Using the above methods its possible to build an application which places an emphasis on time ordered data (such as a group DM or news feed). Or you can use the time ordered nature of the data as a natural pagination for larger queries where you may wish to aggregate data over a given time period and then perform some further computations over it.