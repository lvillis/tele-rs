use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn write_text_if_changed(
    path: &Path,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(existing) if existing == content => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    write_text_atomic(path, content)
}

fn write_text_atomic(path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let parent = parent_dir(path);
    fs::create_dir_all(parent)?;

    for attempt in 0..16 {
        let temp_path = temp_path(path, attempt)?;
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp_path)
        {
            Ok(mut file) => {
                let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                    file.write_all(content.as_bytes())?;
                    file.sync_all()?;
                    Ok(())
                })();
                if let Err(error) = result {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error);
                }

                drop(file);
                if let Err(error) = replace_file(&temp_path, path) {
                    let _ = fs::remove_file(&temp_path);
                    return Err(error.into());
                }
                sync_parent_dir(parent)?;

                return Ok(());
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }

    Err(format!("failed to allocate temporary file for `{}`", path.display()).into())
}

#[cfg(not(windows))]
fn replace_file(temp_path: &Path, path: &Path) -> io::Result<()> {
    fs::rename(temp_path, path)
}

#[cfg(windows)]
fn replace_file(temp_path: &Path, path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    fs::rename(temp_path, path)
}

#[cfg(unix)]
fn sync_parent_dir(parent: &Path) -> io::Result<()> {
    fs::File::open(parent)?.sync_all()
}

#[cfg(not(unix))]
fn sync_parent_dir(_parent: &Path) -> io::Result<()> {
    Ok(())
}

fn parent_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn temp_path(path: &Path, attempt: usize) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let parent = parent_dir(path);
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("path has no file name: {}", path.display()))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());

    let mut temp_name = OsString::from(".");
    temp_name.push(file_name);
    temp_name.push(format!(".tmp-{}-{nonce}-{attempt}", std::process::id()));

    Ok(parent.join(temp_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u128, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!(
            "tele-codegen-{name}-{}-{nonce}",
            std::process::id()
        ))
    }

    #[test]
    fn writes_and_replaces_existing_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("write-replace");
        fs::create_dir_all(&root)?;
        let path = root.join("generated.rs");
        fs::write(&path, "old")?;

        write_text_if_changed(&path, "new")?;

        assert_eq!(fs::read_to_string(&path)?, "new");
        let mut temp_files = 0;
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with(".generated.rs.tmp-")
            {
                temp_files += 1;
            }
        }
        assert_eq!(temp_files, 0);

        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn creates_parent_directories() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("write-create-parent");
        let path = root.join("nested").join("generated.rs");

        write_text_if_changed(&path, "new")?;

        assert_eq!(fs::read_to_string(&path)?, "new");

        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn keeps_existing_file_when_content_matches() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("write-unchanged");
        fs::create_dir_all(&root)?;
        let path = root.join("generated.rs");
        fs::write(&path, "same")?;
        let before = fs::metadata(&path)?.modified()?;

        write_text_if_changed(&path, "same")?;

        assert_eq!(fs::read_to_string(&path)?, "same");
        assert_eq!(fs::metadata(&path)?.modified()?, before);

        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn reports_existing_directory_as_read_error() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("write-directory-target");
        fs::create_dir_all(&root)?;
        let path = root.join("generated.rs");
        fs::create_dir_all(&path)?;

        let result = write_text_if_changed(&path, "new");

        assert!(result.is_err());
        assert!(path.is_dir());

        let _ = fs::remove_dir_all(root);
        Ok(())
    }
}
