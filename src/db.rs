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

pub async fn open(db_path: Arc<str>) -> Result<sled::Db, ()> {
    match sled::open(db_path.to_string()) {
        Err(_) => return Err(()),
        Ok(db) => Ok(db),
    }
}

pub async fn get_all_ids_and_names(
    db: sled::Db,
) -> Result<Vec<Upload>, &'static str> {
    let mut entries = vec![];

    let id_to_sha   = match db.open_tree(b"id_to_sha") {
        Ok(tree) => tree,
        Err(_) => return Err("failed to open id->sha mapping"),
    };
    let sha_to_orig = match db.open_tree(b"sha_to_orig") {
        Ok(tree) => tree,
        Err(_) => return Err("failed to open sha->orig mapping"),
    };

    for tuple in id_to_sha.iter() {
        let (id_ivec, sha_ivec) = match tuple {
            Err(_) => {
                eprintln!("unexpected error while iterating over db entries");
                continue
            },
            Ok((i, s)) => (i,s),
        };

        let id = match from_utf8(&id_ivec) {
            Ok(s) => s,
            _ => {
                eprintln!("failed to convert ivec to utf8 string");
                continue
            },
        };

        let sha = match from_utf8(&sha_ivec) {
            Ok(s) => s,
            _ => {
                eprintln!("failed to convert ivec to utf8 string");
                continue
            },
        };

        let orig_ivec = match sha_to_orig.get(&sha_ivec) {
            Ok(Some(o)) => o,
            Ok(None) => {
                entries.push(Upload {
                    id: id.to_owned(),
                    checksum: sha.to_owned(),
                    orig_name: None,
                });
                continue
            },
            Err(_) => {
                eprintln!("unexpected error looking up original filename for id {}", id);
                entries.push(Upload {
                    id: id.to_owned(),
                    checksum: sha.to_owned(),
                    orig_name: None,
                });
                continue
            }
        };

        let orig = match from_utf8(&orig_ivec) {
            Ok(s) => s,
            _ => {
                eprintln!("failed to convert ivec to utf8 string");
                continue
            },
        };

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
) -> Result<(String, String), &'static str> {
    let id_to_sha   = match db.open_tree(b"id_to_sha") {
        Ok(tree) => tree,
        Err(_) => return Err("failed to open id->sha mapping"),
    };
    let sha_to_orig = match db.open_tree(b"sha_to_orig") {
        Ok(tree) => tree,
        Err(_) => return Err("failed to open sha->orig mapping"),
    };
    let sha256_ivec = match id_to_sha.get(short_id) {
        Ok(Some(ivec)) => ivec,
        _ => return Err("failed to get sha256 from db tree"),
    };
    let sha256 = match from_utf8(sha256_ivec.as_ref()) {
        Ok(s) => s,
        _ => return Err("failed to convert ivec to utf8 string"),
    };
    let orig_ivec = match sha_to_orig.get(sha256.as_bytes()) {
        Ok(Some(ivec)) => ivec,
        _ => return Err("failed to get original name from db tree"),
    };
    let orig = match from_utf8(orig_ivec.as_ref()) {
        Ok(s) => s,
        _ => return Err("failed to convert ivec to utf8 string"),
    };

    Ok((sha256.to_owned(), orig.to_owned()))
}
pub async fn try_get_sha_for_orig(
    db: sled::Db,
    filename: &[u8]
) -> Result<String, &'static str> {
    let orig_to_sha = db.open_tree(b"orig_to_sha").expect("failed to open");
    let sha256_ivec = match orig_to_sha.get(filename) {
        Ok(Some(s)) => s,
        _ => return Err("unknown filename"),
    };
    match std::str::from_utf8(sha256_ivec.as_ref()) {
        Ok(sha256) => return Ok(String::from(sha256)),
        Err(_) => return Err("failed to convert db value to string"),
    }
}

pub async fn try_add_sha_orig(db: sled::Db, sha256: &str, orig: &str) -> Result<(), &'static str> {
    let sha_to_orig = db.open_tree(b"sha_to_orig").expect("failed to open");
    let orig_to_sha = db.open_tree(b"orig_to_sha").expect("failed to open");
    println!("adding {} with orig name {}", sha256, orig);
    if let Err(_) = sha_to_orig.insert(sha256.as_bytes(), orig.as_bytes()) {
        return Err("failed to add sha->orig mapping");
    }
    if let Err(_) = orig_to_sha.insert(orig.as_bytes(), sha256.as_bytes()) {
        return Err("failed to add orig->sha mapping");
    }
    Ok(())
}

pub async fn try_get_new_shortid(db: sled::Db, sha256: &str) -> Result<String, &'static str> {
    let sha_to_id = db.open_tree(b"sha_to_id").expect("failed to open");
    let id_to_sha = db.open_tree(b"id_to_sha").expect("failed to open");

    if let Ok(a) = (&sha_to_id, &id_to_sha)
        .transaction(|(tx_sha_id, tx_id_sha)| {
            // check if file with same hash is already in db
            if let Ok(Some(x)) = tx_sha_id.get(sha256.as_bytes()) {
                // try five times to find an unused short id
                println!("file exists");
                match std::str::from_utf8(x.as_ref()) {
                    Ok(id) => return Ok(String::from(id)),
                    Err(_) => return Err(Abort("failed to convert db value to string")),
                }
            }
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
                if let Err(_) = tx_id_sha.insert(new_id.as_bytes(), sha256.as_bytes()) {
                    println!("failed to insert id->sha mapping");
                    return Err(Abort("could not insert new id->sha mapping"));
                }
                if let Err(_) = tx_sha_id.insert(sha256.as_bytes(), new_id.as_bytes()) {
                    println!("failed to insert sha->id mapping");
                    return Err(Abort("could not insert sha->id mapping"));
                }
                return Ok(new_id);
            };
            println!("failed to find a free short id");
            return Err(Abort("failed to find a free short id"));
        }) {
        return Ok(a);
    }
    println!("failed to execute transaction");
    Err("failed to execute transaction")
}
