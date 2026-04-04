use mongodb::sync::{Client, Collection};
use mongodb::bson::{doc, oid::ObjectId, Binary};
use mongodb::bson::spec::BinarySubtype;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use chrono::Local;
use crate::env;
use once_cell::sync::Lazy;

static MONGO_URI: Lazy<String> = Lazy::new(|| {
    env::get("MONGO_URI")
});

static DB_NAME: Lazy<String> = Lazy::new(|| {
    env::get("DB_NAME")
});

static COLL_NAME: Lazy<String> = Lazy::new(|| {
    env::get("COLL_NAME")
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attempt {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id:        Option<ObjectId>,
    pub timestamp: String,
    pub image:     Binary,
    pub emailed:   bool,
}

fn get_collection() -> Collection<Attempt> {
    let client = Client::with_uri_str(MONGO_URI.as_str())
        .expect("Failed to connect to MongoDB");
    client
        .database(DB_NAME.as_str())
        .collection::<Attempt>(COLL_NAME.as_str())
}

pub fn save_attempt(image_bytes: &[u8]) -> Result<ObjectId, mongodb::error::Error> {
    let coll    = get_collection();
    let attempt = Attempt {
        id:        None,
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        image:     Binary {
            subtype: BinarySubtype::Generic,
            bytes:   image_bytes.to_vec(),
        },
        emailed: false,
    };
    let result = coll.insert_one(attempt, None)?;
    Ok(result.inserted_id.as_object_id().unwrap())
}

pub fn mark_emailed(id: ObjectId) -> Result<(), mongodb::error::Error> {
    let coll = get_collection();
    coll.update_one(
        doc! { "_id": id },
        doc! { "$set": { "emailed": true } },
        None,
    )?;
    Ok(())
}

pub fn get_all_attempts() -> Result<Vec<Attempt>, mongodb::error::Error> {
    let coll    = get_collection();
    let options = FindOptions::builder()
        .sort(doc! { "timestamp": -1 })
        .build();
    let cursor = coll.find(None, options)?;
    cursor.collect()
}

pub fn get_attempt_by_id(id: ObjectId) -> Result<Option<Attempt>, mongodb::error::Error> {
    let coll = get_collection();

    coll.find_one(
        doc! { "_id": id },
        None
    )
}

pub fn get_attempt_count() -> Result<u64, mongodb::error::Error> {
    get_collection().count_documents(None, None)
}

pub fn get_today_count() -> Result<u64, mongodb::error::Error> {
    let today = Local::now().format("%Y-%m-%d").to_string();
    get_collection().count_documents(
        doc! { "timestamp": { "$regex": &today } },
        None,
    )
}

pub fn delete_attempt(id: ObjectId) -> Result<(), mongodb::error::Error> {
    get_collection().delete_one(doc! { "_id": id }, None)?;
    Ok(())
}

pub fn delete_all() -> Result<(), mongodb::error::Error> {
    get_collection().delete_many(doc! {}, None)?;
    Ok(())
}