use anyhow::Context;
use std::fs::File;
use std::path::Path;

fn create_key_file(path: &Path) -> anyhow::Result<File> {
    let f = File::create(path).context("Error creating local key file")?;

    #[cfg(not(target_os = "windows"))]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        f.set_permissions(Permissions::from_mode(0o600))
            .context("Error setting permissions")?;
    }

    Ok(f)
}

pub async fn get_local_key(base_dir: &Path) -> anyhow::Result<Vec<u8>> {
    let session_path_absolute =
        std::fs::canonicalize(base_dir).context("error getting absolute path")?;
    let key_path = session_path_absolute.join("local.key");

    if key_path.exists() && key_path.is_file() {
        return tokio::fs::read(&key_path)
            .await
            .context("Error reading local key file");
    }

    info!("Couldn't find local key file, generating one");

    create_key_file(&key_path).context("Error creating local key file")?;

    let key = pass_domain::crypto::generate_encryption_key();
    tokio::fs::write(key_path, &key)
        .await
        .context("Error writing key")?;

    Ok(key)
}
