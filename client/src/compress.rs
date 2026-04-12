use std::fs::{self, File};
use std::io;
use std::path::Path;

use walkdir::WalkDir;
use zip::ZipArchive;
use zip::write::FileOptions;
use zip::{result::ZipError, write::SimpleFileOptions};

fn zip_dir(src_dir: &Path, dest_file: &Path, options: FileOptions<'_, ()>) -> anyhow::Result<()> {
    let file = File::create(dest_file)?;

    let walkdir = WalkDir::new(src_dir);

    let mut zip = zip::ZipWriter::new(file);

    for entry_result in walkdir.into_iter() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "Error while traversing directory {src_dir:?}: {e}"
                )));
            }
        };
        let path = entry.path();
        let path_stripped = path.strip_prefix(src_dir)?;
        let path_as_string = path_stripped.to_str().map(str::to_owned).ok_or_else(|| {
            anyhow::Error::msg(format!("{:?} is a Non UTF-8 Path", path_stripped.display()))
        })?;

        if path.is_file() {
            println!(
                "adding file {:?} as {:?} ...",
                path.display(),
                path_stripped.display()
            );
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            std::io::copy(&mut f, &mut zip)?;
        } else if !path_stripped.as_os_str().is_empty() {
            println!(
                "adding dir '{}' as '{}' ...",
                path.display(),
                path_stripped.display()
            );
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn zip_file(src_file: &Path, dest_file: &Path, options: FileOptions<'_, ()>) -> anyhow::Result<()> {
    let file = File::create(dest_file)?;
    let mut zip = zip::ZipWriter::new(file);
    let path = src_file;
    let path_stripped = src_file.file_name().unwrap();
    let path_as_string = path_stripped.to_str().map(str::to_owned).ok_or_else(|| {
        anyhow::Error::msg(format!("{:?} is a Non UTF-8 Path", path_stripped.display()))
    })?;
    println!(
        "adding file {:?} as {:?} ...",
        path.display(),
        path_stripped.display()
    );
    zip.start_file(path_as_string, options)?;
    let mut f = File::open(path)?;

    std::io::copy(&mut f, &mut zip)?;
    zip.finish()?;
    Ok(())
}

pub fn compress_archieve(src: &Path, dest_file: &Path) -> anyhow::Result<()> {
    if dest_file.exists() {
        return Err(anyhow::Error::msg(format!(
            "File {} already exists",
            dest_file.display()
        )));
    }

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Zstd)
        .unix_permissions(0o755)
        .large_file(true)
        .compression_level(Some(10));
    if src.is_file() {
        zip_file(src, dest_file, options)
    } else if src.is_dir() {
        zip_dir(src, dest_file, options)
    } else {
        Err(ZipError::FileNotFound.into())
    }
}

pub fn decompress_archieve(src: &Path, dest_dir: &Path) -> anyhow::Result<()> {
    if !dest_dir.exists() {
        std::fs::create_dir_all(dest_dir)?;
    }

    if !src.is_file() {
        return Err(ZipError::FileNotFound.into());
    }

    let file = File::open(src)?;
    let mut archieve = ZipArchive::new(file)?;
    for i in 0..archieve.len() {
        let mut file = archieve.by_index(i)?;
        let out_path = dest_dir.join(
            file.enclosed_name()
                .ok_or(anyhow::Error::msg(format!("Error file name in file {}", i)))?,
        );
        println!("{}", &out_path.display());
        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent()
                && !p.exists()
            {
                fs::create_dir_all(p)?;
            }
            let mut outfile = fs::File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
