mod config;
mod db;
mod file;

use config::{Config, SuffixType};

use std::io::prelude::*;
use std::fs::{File, OpenOptions};
use std::sync::Arc;

use futures_core::{Future, Stream};
use futures_util::TryStreamExt;
use hyper::body::Body;
use mime::Mime;
use mpart_async::server::MultipartStream;
use tokio_stream::wrappers::UnixListenerStream;
use uuid::Uuid;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::Signals;
use warp::{Filter, Rejection};
use warp::http::{Response,StatusCode};

#[tokio::main]
async fn main() {
    let config = Config::init();
    config.print();

    let db = match db::open(config.db_path.clone()).await {
        Ok(db) => db,
        Err(_) => {
            eprintln!("Error: failed to open database {}", config.db_path);
            std::process::exit(1);
        }
    };

    let listener = match file::setup_dirs_get_listener(&config.socket_path).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let db_up = db.clone();
    let config_up = config.clone();
    let upload = warp::path("up")
        .and(warp::path::end())
        .and(warp::post())
        // limit upload size
        .and(warp::body::content_length_limit(500_000_000))
        .and(warp::header::<Mime>("content-type"))
        .and(warp::body::stream())
        .and_then(move |mime,body| {
            handle_upload(mime, body, db_up.clone(), config_up.clone())
        });

    let too_large = warp::path("up")
        .and(warp::path::end())
        .and(warp::post())
        .and(warp::header::<Mime>("content-type"))
        .map(|_| {
            Response::builder()
                .status(StatusCode::PAYLOAD_TOO_LARGE)
                .body("Payload too large\n")
        });

    let db_id = db.clone();
    let config_id = config.clone();
    let download_id = warp::get()
        .and(warp::path("f"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(move |id| {
            construct_response_for_id(id, config_id.clone(), db_id.clone())
        });

    let config_state = config.clone();
    let db_state = db.clone();
    let state = warp::get()
        .and(warp::path("state"))
        .and(warp::path::end())
        .and_then(move || {
            construct_state_response(config_state.clone(), db_state.clone())
        });


    let db_orig = db.clone();
    let download_orig = warp::get()
        .and(warp::path("d"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(move |filename| {
            construct_response_for_filename(filename, db_orig.clone())
        });

    let landing_page = warp::get()
        .and(warp::path::end())
        .map(|| {
            Response::builder()
                .status(StatusCode::OK)
                .body("Urlnao - \
                    Upload service for file sharing with weechat-android\n")
        });

    let reject = warp::any()
        .map(|| {
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Bad Request\n")
        });

    let routes = landing_page
        .or(download_id)
        .or(download_orig)
        .or(upload)
        .or(too_large)
        .or(state)
        .or(reject);

    let incoming = UnixListenerStream::new(listener);

    let server = tokio::spawn(async move {
        warp::serve(routes).run_incoming(incoming).await;
    });

    let sigwait = tokio::spawn(async move {
        term_signal().await
    });

    futures::future::select(server, sigwait).await;
    cleanup(config);
}

fn cleanup(config: Config) {
    if let Err(_) = std::fs::remove_file(config.socket_path.to_string()) {
        eprintln!("failed to cleanup socket {}", config.socket_path);
    }
}

async fn term_signal() {
    let mut signals = Signals::new(TERM_SIGNALS).unwrap();

    for _ in &mut signals {
        eprintln!("Received signal, terminating.");
        break;
    }
}

fn append_temp_suffix(s: &str) -> String {
    format!("{}.part", s)
}

fn new_random_uuid() -> String {
    Uuid::new_v4().to_string()
}

async fn construct_state_response(
    config: Config,
    db: sled::Db
) -> Result<http::Response<String>, Rejection> {
    let mut response = vec![];

    response.push("<!doctype html>\n\
                   <head>\n\
                     <meta charset=\"utf-8\">\n\
                     <title>Urlnao</title>\n\
                   </head>\n\
                   <body>".to_owned());

    let entries = match db::get_all_ids_and_names(db).await {
        Ok(e) => e,
        Err(_) => return Err(warp::reject::not_found()),
    };

    response.push(format!("<p>Urlnao currently has {} upload(s):</p>\n<ul>", entries.len()));

    for upload in entries {
        match upload.orig_name {
            Some(orig_name) => response.push(format!(
                    "<pre><li><a href=\"{}\">checksum: {} (filename: {})</a></li></pre>",
                    config.prepend_url(SuffixType::ShortID, &upload.id),
                    upload.checksum,
                    orig_name)),
            None => response.push(format!(
                    "<pre><li><a href=\"{}\">checksum: {} (no filename)</a></li></pre>",
                    config.prepend_url(SuffixType::ShortID, &upload.id),
                    upload.checksum)),
        }
    }

    response.push("</ul>\n</body>\n".to_owned());

    match Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(response.join("\n")) {
            Err(_) => Err(warp::reject::not_found()),
            Ok(response) => Ok(response),
    }
}

async fn construct_response_for_id(
    short_id: String,
    config: Config,
    db: sled::Db
) -> Result<http::Response<Body>, Rejection> {
    let (_, orig) = match db::try_get_sha_and_orig(db, short_id.as_bytes()).await {
        Ok(t)  => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(warp::reject::not_found());
        },
    };
    let response = match Response::builder()
                    .status(StatusCode::MOVED_PERMANENTLY)
                    .header("Location", config.prepend_url(SuffixType::FileName, &orig))
                    .body(Body::empty()) {
        Ok(r) => r,
        Err(_) => return Err(warp::reject::not_found()),
    };
    Ok(response)
}

async fn construct_response_for_filename(
    filename: String,
    db: sled::Db
) -> Result<http::Response<Vec<u8>>, Rejection> {
    let sha256 = match db::try_get_sha_for_orig(db, filename.as_bytes()).await {
        Ok(t)  => t,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(warp::reject::not_found());
        },
    };
    let mut file = match File::open(format!("uploads/{}", sha256)) {
        Ok(file) => file,
        Err(_) => {
            eprintln!("Error: file not found");
            return Err(warp::reject::not_found());
        },
    };

    let split = &filename.split(".").collect::<Vec<&str>>();
    let ext = split.last();
    let content_type = match *ext.unwrap() {
        "bmp"  => "image/bmp",
        "gif"  => "image/gif",
        "jpeg" |
        "jpg"  => "image/jpeg",
        "json" => "application/json",
        "mp3"  => "audio/mpeg",
        "mp4"  => "video/mp4",
        "mpeg" => "video/mpeg",
        "pdf"  => "application/pdf",
        "png"  => "image/png",
        "svg"  => "image/svg+xml",
        "txt"  => "text/plain",
        "webm" => "video/webm",
        "webp" => "image/webp",
        _      => "application/octet-stream",
    };
    let content_disposition = match content_type {
        "application/json" |
        "application/octet-stream" |
        "application/pdf" => {
            format!("attachment; filename={}", filename)
        },
        _ => "inline".to_string(),
    };

    let mut data = vec![];
    let response = match file.read_to_end(&mut data) {
        Ok(_) => {
            match Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", content_type)
                    .header("Content-Disposition", content_disposition)
                    .body(data) {
                Ok(response) => response,
                Err(_) => {
                    eprintln!("failed to build response");
                    return Err(warp::reject::not_found());
                },
            }
        },
        Err(_) => {
            eprintln!("failed to read file");
            return Err(warp::reject::not_found())
        }
    };

    Ok(response)
}

async fn create_upload_tasks(
    new_files: Vec<file::FileInfo>,
    db: sled::Db
) -> Vec<impl Future<Output = Option<String>>> {
    let mut tasks = vec![];

    for file_info in new_files {
        let orig_name = file_info.original_filename;
        let name = append_temp_suffix(&file_info.uuid);
        let db = db.clone();
        tasks.push(futures::future::lazy(|_| async move {
            let checksum = file::get_sha256_of_file(&name);

            let sha256 = match checksum.await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return None;
                },
            };
            if let Err(_) = file::try_move_file(&name, &sha256).await {
                eprintln!("Error: failed to rename file");
                return None;
            }
            let short_id = match db::try_get_new_shortid(db.clone(), &sha256).await {
                Ok(s) => s,
                Err(_) => return None,
            };
            if let Err(e) = db::try_add_sha_orig(db, &sha256, &orig_name).await {
                eprintln!("Error: {}", e);
                return None;
            }
            Some(short_id)
        }).await);
    }

    tasks
}

async fn handle_upload(
    mime: Mime,
    body: impl Stream<Item = Result<impl bytes::Buf, warp::Error>> + Unpin,
    db: sled::Db,
    config: Config,
) -> Result<impl warp::Reply, Rejection> {
    let boundary = mime
        .get_param("boundary")
        .map(|v| v.to_string())
        .ok_or_else(|| {
            warp::reject::not_found()
        })?;

    let mut parts = MultipartStream::new(
        boundary,
        body.map_ok(|mut buf| buf.copy_to_bytes(buf.remaining())),
    );

    let mut file_index = 0;
    let mut new_files: Vec<file::FileInfo> = Vec::new();

    while let Ok(Some(mut form_field)) = parts.try_next().await {
        match form_field.filename() {
            Ok(filename) => {
                let uuid = Arc::from(new_random_uuid());
                let original_filename = Arc::from(filename);

                new_files.push(file::FileInfo {
                    uuid,
                    original_filename,
                });
            },
            Err(_) => {
                println!("Warn: client did not send filename, ignoring part");
                continue
            },
        }

        let name = append_temp_suffix(&new_files[file_index].uuid);
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(name.as_str()) {
            while let Ok(Some(bytes)) = form_field.try_next().await {
                let _ = file.write(bytes.to_vec().as_ref());
            }
            file_index += 1;
        } else {
          println!("error creating temporary file {}", name);
          return Err(warp::reject::not_found());
        };

    }

    let tasks = create_upload_tasks(new_files, db).await;

    let maybe_urls = futures::future::join_all(tasks).await;

    let mut response = vec![];
    for url in maybe_urls {
        match url {
            Some(short_id) => response.push(config.prepend_url(SuffixType::ShortID, &short_id)),
            None => response.push(String::from("upload failed")),
        }
    }

    Ok(format!("{}\n", response.join("\n")))
}
