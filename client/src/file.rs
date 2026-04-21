pub mod archive;
pub mod compress;
pub mod crypto;

#[cfg(test)]
mod tests {
    use std::{io::Write, path::Path};

    use fake::Fake;
    use tempfile::TempDir;
    use uuid::Uuid;
    use walkdir::WalkDir;

    use crate::file::crypto::generate_aes_keys;

    use super::*;

    fn generate_test_dir() -> anyhow::Result<TempDir> {
        let tmp_dir = tempfile::tempdir()?;
        let mut tmp_file = std::fs::File::create(tmp_dir.path().join(Uuid::new_v4().to_string()))?;
        let info: Vec<u8> = fake::Faker.fake();
        tmp_file.write_all(&info)?;
        Ok(tmp_dir)
    }

    fn is_dir_equal(dir1: &Path, dir2: &Path) -> bool {
        let walker = WalkDir::new(dir1);
        for entry in walker.into_iter() {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_stripped = path.strip_prefix(dir1).unwrap();
            if path.is_file() {
                let dir1_content = std::fs::read(path).unwrap();
                if let Ok(dir2_content) = std::fs::read(dir2.join(path_stripped)) {
                    if dir1_content != dir2_content {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            if path.is_dir() {
                let path2 = dir2.join(path_stripped);
                if !path2.exists() || !path2.is_dir() {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn test_compress_and_decompress() {
        let test_dir = generate_test_dir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let out_file = output_dir.path().join(Uuid::new_v4().to_string());
        compress::compress_archive(test_dir.path(), &out_file).unwrap();
        let decompress_dir = &output_dir.path().join(Uuid::new_v4().to_string());
        compress::decompress_archive(&out_file, decompress_dir).unwrap();
        assert!(is_dir_equal(test_dir.path(), decompress_dir));
        test_dir.close().unwrap();
        output_dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_pack_and_unpack() {
        let test_dir = generate_test_dir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let out_file = output_dir.path().join(Uuid::new_v4().to_string());
        let key = &generate_aes_keys();
        archive::pack_archive(test_dir.path(), &out_file, key)
            .await
            .unwrap();
        let unpack_dir = &output_dir.path().join(Uuid::new_v4().to_string());
        archive::unpack_archive(&out_file, unpack_dir, key)
            .await
            .unwrap();
        assert!(is_dir_equal(test_dir.path(), unpack_dir));
        test_dir.close().unwrap();
        output_dir.close().unwrap();
    }
}
