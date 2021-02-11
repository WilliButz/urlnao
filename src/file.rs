use crate::util;

use sha2::Digest;
use tokio::net::UnixListener;

use std::fs::{
    File,
    Permissions,
};
use std::sync::Arc;
use std::os::unix::fs::PermissionsExt;

pub struct FileInfo {
    pub original_filename: Arc<str>,
    pub uuid:              Arc<str>,
}

pub async fn setup_dirs_get_listener(socket_path: &Arc<str>) -> Result<UnixListener, String> {
    let listener = UnixListener::bind(socket_path.to_string())
        .map_err(|e| e.to_string())?;

    let socket_permissions = Permissions::from_mode(0o660);

    std::fs::set_permissions(socket_path.to_string(), socket_permissions)
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all("uploads")
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all("tmp")
        .map_err(|e| e.to_string())?;

    Ok(listener)
}

pub async fn get_sha256_of_file<'a>(path: &str) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;

    let mut sha256 = sha2::Sha256::new();

    let n = std::io::copy(&mut file, &mut sha256).map_err(|e| e.to_string())?;

    assert!(n > 0); // file should never be empty

    Ok(format!("{:x}", sha256.finalize()))
}

pub async fn try_move_to_uploads(from: &str, to: &str) -> Result<(), String> {
    let target = util::prepend_upload_dir(to);

    if std::fs::metadata(&target).is_ok() {
        eprintln!("Warning: file {} exists, not overwriting", to);
        return Ok(());
    }

    std::fs::rename(&from, &target)
        .map_err(|e| {
            format!("failed to rename file: {}: {} -> {}", e, from, target)
        })?;

    Ok(())
}
