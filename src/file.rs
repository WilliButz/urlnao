use sha2::Digest;
use std::sync::Arc;
use std::fs::{
    File,
    Permissions,
    rename
};
use tokio::net::UnixListener;
use std::os::unix::fs::PermissionsExt;

pub struct FileInfo {
    pub original_filename: Arc<str>,
    pub uuid:              Arc<str>,
}

pub async fn setup_dirs_get_listener(socket_path: &Arc<str>) -> Result<UnixListener, String> {
    let listener = match UnixListener::bind(socket_path.to_string()) {
        Ok(listener) => listener,
        Err(e) => {
            return Err(format!("failed to bind to socket: {}", e));
        },
    };

    let socket_permissions = Permissions::from_mode(0o660);
    if let Err(e) = std::fs::set_permissions(socket_path.to_string(), socket_permissions) {
        return Err(format!("failed to socket permissions: {}", e));
    }

    if let Err(e) = std::fs::create_dir_all("uploads") {
        return Err(format!("failed to create directory 'uploads': {}", e));
    }
    if let Err(e) = std::fs::create_dir_all("tmp") {
        return Err(format!("failed to create directory 'tmp': {}", e));
    }
    Ok(listener)
}

pub async fn get_sha256_of_file(path: &str) -> Result<String, &'static str> {
    if let Ok(mut file) = File::open(path) {
        let mut sha256 = sha2::Sha256::new();
        return match std::io::copy(&mut file, &mut sha256) {
            Ok(n) if n > 0 => Ok(format!("{:x}", sha256.finalize())),
            _ => Err("failed to calculate checksum"),
        };
    }
    Err("failed to open file")
}

pub async fn try_move_file(from: &str, to: &str) -> Result<(), &'static str> {
    match rename(from, format!("uploads/{}", to)) {
        Ok(_) => Ok(()),
        _ => Err("failed to move file")
    }
}
