use rand::{
    prelude::*,
    distributions::Alphanumeric
};
use sled::{
    Transactional,
    transaction::ConflictableTransactionError::Abort
};

use std::str::from_utf8;
use std::sync::Arc;

pub struct Upload {
    pub id:        String,
    pub checksum:  String,
    pub orig_name: Option<String>,
}

pub async fn open(db_path: Arc<str>) -> Result<sled::Db, String> {
    let db = sled::open(db_path.to_string())
        .map_err(|e| e.to_string())?;

    Ok(db)
}

pub async fn get_all_ids_and_names(
    db: sled::Db,
) -> Result<Vec<Upload>, String> {
    let mut entries = vec![];

    let id_to_sha = db.open_tree(b"id_to_sha")
        .map_err(|e| e.to_string())?;

    let sha_to_orig = db.open_tree(b"sha_to_orig")
        .map_err(|e| e.to_string())?;

    for tuple in id_to_sha.iter() {
        let (id_ivec, sha_ivec) = tuple
            .map_err(|e| e.to_string())?;

        let id = from_utf8(&id_ivec)
            .map_err(|e| e.to_string())?;

        let sha = from_utf8(&sha_ivec)
            .map_err(|e| e.to_string())?;

        let query_result = sha_to_orig.get(&sha_ivec)
            .map_err(|e| e.to_string())?;

        let orig_ivec = match query_result {
           Some(ivec) => ivec,
           None => {
               entries.push(Upload {
                   id: id.to_owned(),
                   checksum: sha.to_owned(),
                   orig_name: None,
               });
               continue
           },
        };

        let orig = from_utf8(&orig_ivec)
            .map_err(|e| e.to_string())?;

        entries.push(Upload {
            id: id.to_owned(),
            checksum: sha.to_owned(),
            orig_name: Some(orig.to_owned()),
        });
    }

    Ok(entries)
}

pub async fn try_get_sha_and_orig(
    db: sled::Db,
    short_id: &[u8]
) -> Result<(String, String), String> {
    let id_to_sha = db.open_tree(b"id_to_sha")
        .map_err(|e| e.to_string())?;

    let sha_to_orig = db.open_tree(b"sha_to_orig")
        .map_err(|e| e.to_string())?;

    let query_result = id_to_sha.get(short_id)
        .map_err(|e| e.to_string())?;

    let sha256_ivec = match query_result {
        Some(ivec) => ivec,
        None => return Err("failed to find checksum in db tree".to_string()),
    };

    let sha256 = from_utf8(&sha256_ivec)
        .map_err(|e| e.to_string())?;

    let query_result = sha_to_orig.get(&sha256_ivec)
        .map_err(|e| e.to_string())?;

    let orig_ivec = match query_result {
        Some(ivec) => ivec,
        None => return Err("failed to find original filename in db tree".to_string()),
    };

    let orig = from_utf8(&orig_ivec)
        .map_err(|e| e.to_string())?;

    Ok((sha256.to_owned(), orig.to_owned()))
}

pub async fn try_get_sha_for_orig(
    db: sled::Db,
    filename: &[u8]
) -> Result<String, String> {
    let orig_to_sha = db.open_tree(b"orig_to_sha")
        .map_err(|e| e.to_string())?;

    let query_result = orig_to_sha.get(filename)
        .map_err(|e| e.to_string())?;

    let sha256_ivec = match query_result {
        Some(ivec) => ivec,
        None => return Err("unknown filename".to_string()),
    };

    let sha256 = from_utf8(&sha256_ivec)
        .map_err(|e| e.to_string())?;

    Ok(sha256.to_owned())
}

pub async fn try_add_sha_orig(db: sled::Db, sha256: &str, orig: &str) -> Result<(), String> {
    let sha_to_orig = db.open_tree(b"sha_to_orig")
        .map_err(|e| e.to_string())?;

    let orig_to_sha = db.open_tree(b"orig_to_sha")
        .map_err(|e| e.to_string())?;

    println!("adding {} with orig name {}", sha256, orig);

    (&sha_to_orig, &orig_to_sha)
        .transaction(|(tx_sha_orig, tx_orig_sha)| {
            tx_sha_orig.insert(sha256.as_bytes(), orig.as_bytes())?;

            tx_orig_sha.insert(orig.as_bytes(), sha256.as_bytes())?;

            Ok(())

        })
    .map_err(|e: sled::transaction::TransactionError<&str>| {
        e.to_string()
    })?;

    Ok(())
}

pub async fn try_get_new_shortid(db: sled::Db, sha256: &str) -> Result<String, String> {
    let sha_to_id = db.open_tree(b"sha_to_id")
        .map_err(|e| e.to_string())?;

    let id_to_sha = db.open_tree(b"id_to_sha")
        .map_err(|e| e.to_string())?;

    let new_id = (&sha_to_id, &id_to_sha)
        .transaction(|(tx_sha_id, tx_id_sha)| {
            // check if file with same hash is already in db
            let query_result = tx_sha_id.get(sha256.as_bytes())?;

            if let Some(x) = query_result {
                let id = from_utf8(&x).map_err(|_| {
                    Abort("failed to convert query result to string")
                })?;

                println!("Info: reusing existing ID for duplicate upload: {}", id);
                return Ok(id.to_string());
            }

            // try five times to find an unused short id
            for _ in 0..5 {
                // generate a 3 to 8 character long short id
                let new_id: String = std::iter::repeat(())
                    .map(|()| thread_rng().sample(Alphanumeric))
                    .map(char::from)
                    .take(thread_rng().gen_range(3..=8))
                    .collect();
                // check if short id is already in use
                if let Ok(Some(_)) = tx_id_sha.get(new_id.as_bytes()) {
                    continue
                }
                // try to insert mapping sha256 -> new_id
                tx_id_sha.insert(new_id.as_bytes(), sha256.as_bytes())?;
                tx_sha_id.insert(sha256.as_bytes(), new_id.as_bytes())?;

                return Ok(new_id);
            };
            return Err(Abort("failed to find a free short id"));
        }).map_err(|e: sled::transaction::TransactionError<&str>| {
          e.to_string()
      })?;

    Ok(new_id)
}
