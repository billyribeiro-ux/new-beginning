//! `RecordingObjectStore` — in-memory fake for tests / evidence runs that
//! don't have R2 creds. Stores bytes in a HashMap and returns deterministic
//! presigned URLs.

use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use url::Url;

use crate::api::{ObjectStore, StoreError};

#[derive(Clone, Default)]
pub struct RecordingObjectStore {
    inner: Arc<Mutex<HashMap<String, (Bytes, String)>>>,
}

impl RecordingObjectStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_raw(&self, key: &str) -> Option<(Bytes, String)> {
        self.inner.lock().unwrap().get(key).cloned()
    }

    pub fn keys(&self) -> Vec<String> {
        let mut k: Vec<String> = self.inner.lock().unwrap().keys().cloned().collect();
        k.sort();
        k
    }
}

#[async_trait]
impl ObjectStore for RecordingObjectStore {
    fn presigned_get(&self, key: &str, ttl: Duration) -> Result<Url, StoreError> {
        // Deterministic URL so tests can assert it. Encodes the TTL for
        // visibility but the fake doesn't enforce expiry.
        let secs = ttl.as_secs();
        let s = format!("https://r2.test/fake/{key}?ttl={secs}");
        Url::parse(&s).map_err(|e| StoreError::Bucket(e.to_string()))
    }

    async fn put_object(
        &self,
        key: &str,
        bytes: Bytes,
        content_type: &str,
    ) -> Result<(), StoreError> {
        self.inner
            .lock()
            .unwrap()
            .insert(key.to_string(), (bytes, content_type.to_string()));
        Ok(())
    }

    async fn head_object(&self, key: &str) -> Result<(), StoreError> {
        if self.inner.lock().unwrap().contains_key(key) {
            Ok(())
        } else {
            Err(StoreError::NotFound)
        }
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.inner.lock().unwrap().remove(key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_then_head_then_delete() {
        let s = RecordingObjectStore::new();
        s.put_object("a/b.pdf", Bytes::from_static(b"hi"), "application/pdf")
            .await
            .unwrap();
        s.head_object("a/b.pdf").await.unwrap();
        s.delete("a/b.pdf").await.unwrap();
        assert!(matches!(
            s.head_object("a/b.pdf").await.unwrap_err(),
            StoreError::NotFound
        ));
    }

    #[test]
    fn presigned_get_encodes_key_and_ttl() {
        let s = RecordingObjectStore::new();
        let u = s
            .presigned_get("invoices/x/y.pdf", Duration::from_secs(300))
            .unwrap();
        assert!(u.as_str().contains("invoices/x/y.pdf"));
        assert!(u.as_str().contains("ttl=300"));
    }
}
