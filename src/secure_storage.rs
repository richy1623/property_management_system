use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use chrono::{Datelike, NaiveDate};
use std::path::{Path, PathBuf};
use tokio::fs::{self as tokio_fs, File};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::model::Error;

pub struct SecureStorage {
    cipher: Aes256Gcm,
    root_path: PathBuf,
}

impl SecureStorage {
    pub fn new(root_path: impl AsRef<Path>, key_bytes: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key_bytes));
        Self {
            cipher,
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    pub async fn store_file(
        &self,
        file_name: &str,
        content: &[u8],
        date: NaiveDate,
    ) -> Result<PathBuf, Error> {
        let dir = self.root_path.join(format!(
            "{}/{:02}/{:02}",
            date.year_ce().1,
            date.month(),
            date.day()
        ));
        tokio_fs::create_dir_all(&dir).await?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let encrypted = self.cipher.encrypt(&nonce, content)?;

        let mut file_path = dir.join(file_name);
        file_path.set_extension("enc");

        let mut file = File::create(&file_path).await?;
        file.write_all(&nonce).await?;
        file.write_all(&encrypted).await?;
        file.flush().await?;

        Ok(file_path)
    }

    pub async fn read_file(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
        let mut file = File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        if buffer.len() < 12 {
            return Err(Error::InternalError(
                "File too short to contain nonce".to_string(),
            ));
        }

        let (nonce_bytes, cipher_text) = buffer.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let decrypted = self
            .cipher
            .decrypt(nonce, cipher_text)
            .map_err(Error::from)?;

        Ok(decrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio;

    #[tokio::test]
    async fn test_store_and_read_file_roundtrip() {
        let temp_dir = tempdir().unwrap();
        let key = [42u8; 32];

        let storage = SecureStorage::new(temp_dir.path(), &key);

        let content = b"confidential data";
        let file_name = "secret.txt";
        let date = NaiveDate::from_ymd_opt(2025, 5, 25).unwrap();

        let file_path = storage
            .store_file(file_name, content, date)
            .await
            .expect("storing file should succeed");

        assert!(file_path.exists());

        let decrypted = storage
            .read_file(&file_path)
            .await
            .expect("reading file should succeed");

        assert_eq!(decrypted, content);
    }

    #[tokio::test]
    async fn test_read_file_fails_on_short_file() {
        let temp_dir = tempdir().unwrap();
        let key = [42u8; 32];
        let storage = SecureStorage::new(temp_dir.path(), &key);

        let file_path = temp_dir.path().join("corrupt.enc");
        tokio::fs::write(&file_path, b"too short").await.unwrap();

        let result = storage.read_file(&file_path).await;
        assert!(matches!(result, Err(Error::InternalError(msg)) if msg.contains("too short")));
    }

    #[tokio::test]
    async fn test_read_file_fails_on_invalid_encryption() {
        let temp_dir = tempdir().unwrap();
        let key = [42u8; 32];
        let storage = SecureStorage::new(temp_dir.path(), &key);

        let file_path = temp_dir.path().join("invalid.enc");

        // 12-byte nonce + random data that won’t decrypt properly
        let mut bad_data = vec![0u8; 12];
        bad_data.extend_from_slice(b"invalid_key");
        tokio::fs::write(&file_path, &bad_data).await.unwrap();

        let result = storage.read_file(&file_path).await;
        assert!(matches!(result, Err(Error::InternalError(_))));
    }

    #[tokio::test]
    async fn test_store_file_creates_correct_directory_structure() {
        let temp_dir = tempdir().unwrap();
        let key = [42u8; 32];
        let storage = SecureStorage::new(temp_dir.path(), &key);

        let date = NaiveDate::from_ymd_opt(2025, 1, 9).unwrap();
        let _ = storage
            .store_file("example.txt", b"hello world", date)
            .await
            .unwrap();

        let expected_path = temp_dir.path().join("2025").join("01").join("09");

        assert!(expected_path.exists());
    }
}
