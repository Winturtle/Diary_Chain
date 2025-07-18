use chrono::Utc;
use serde::{Serialize, Deserialize};
use chrono_tz::Asia::Taipei;

#[derive(Serialize, Deserialize, Clone)]
pub struct DiaryMetadata {
    pub filename: String,
    pub hash: String,
    pub timestamp: String,
    pub block_index: u64,
    pub previous_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: String,
    pub previous_hash: String,
    pub data_hash: String,
    pub metadata: DiaryMetadata,
}

pub fn create_block(index: u64, previous_hash: String, data_hash: String, filename: String) -> Block {
    let utc_now = Utc::now();
    let local_time = utc_now.with_timezone(&Taipei);
	let timestamp = local_time.to_rfc3339();

    let metadata = DiaryMetadata {
        filename,
        hash: data_hash.clone(),
        timestamp: timestamp.clone(),
        block_index: index,
        previous_hash: previous_hash.clone(),
    };

    Block {
        index,
        timestamp,
        previous_hash,
        data_hash,
        metadata,
    }
}