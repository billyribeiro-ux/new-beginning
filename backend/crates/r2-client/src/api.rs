//! `ObjectStore` trait + real `R2Client` impl.
//!
//! R2's S3-compatible endpoint:
//!   `https://{account_id}.r2.cloudflarestorage.com`
//! Bucket goes in the path (not the subdomain) — set `path_style` true.
//! Region is always `"auto"`.

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::Client;
use rusty_s3::actions::{DeleteObject, GetObject, PutObject, S3Action};
use rusty_s3::{Bucket, Credentials, UrlStyle};
use secrecy::{ExposeSecret, SecretString};
use std::time::Duration;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("bucket setup: {0}")]
    Bucket(String),
    #[error("upstream {status}: {body}")]
    Upstream { status: u16, body: String },
    #[error("not found")]
    NotFound,
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Returns a short-TTL presigned URL the BFF can hand to the browser.
    fn presigned_get(&self, key: &str, ttl: Duration) -> Result<Url, StoreError>;

    /// Used by workers (PR #11) to upload artifacts that arrive from
    /// outside (Stripe invoice PDFs, data exports).
    async fn put_object(
        &self,
        key: &str,
        bytes: Bytes,
        content_type: &str,
    ) -> Result<(), StoreError>;

    /// HEAD — used by the GET pre-sign endpoint to surface a 404 to the
    /// caller before minting a URL for a non-existent key.
    async fn head_object(&self, key: &str) -> Result<(), StoreError>;

    /// Delete an object (account-deletion path, PR #16).
    async fn delete(&self, key: &str) -> Result<(), StoreError>;
}

#[derive(Clone)]
pub struct R2Client {
    http: Client,
    bucket: Bucket,
    creds: Credentials,
}

impl R2Client {
    pub fn new(
        endpoint_url: &str,
        bucket_name: &str,
        access_key_id: SecretString,
        secret_access_key: SecretString,
    ) -> Result<Self, StoreError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(5))
            .build()?;
        let url = Url::parse(endpoint_url).map_err(|e| StoreError::Bucket(e.to_string()))?;
        let bucket = Bucket::new(
            url,
            UrlStyle::Path,
            bucket_name.to_string(),
            "auto".to_string(),
        )
        .map_err(|e| StoreError::Bucket(e.to_string()))?;
        let creds = Credentials::new(
            access_key_id.expose_secret(),
            secret_access_key.expose_secret(),
        );
        Ok(Self {
            http,
            bucket,
            creds,
        })
    }
}

#[async_trait]
impl ObjectStore for R2Client {
    fn presigned_get(&self, key: &str, ttl: Duration) -> Result<Url, StoreError> {
        let action = GetObject::new(&self.bucket, Some(&self.creds), key);
        Ok(action.sign(ttl))
    }

    async fn put_object(
        &self,
        key: &str,
        bytes: Bytes,
        content_type: &str,
    ) -> Result<(), StoreError> {
        let action = PutObject::new(&self.bucket, Some(&self.creds), key);
        let url = action.sign(Duration::from_secs(60));
        let resp = self
            .http
            .put(url)
            .header("content-type", content_type)
            .body(bytes)
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(StoreError::Upstream {
                status: status.as_u16(),
                body,
            })
        }
    }

    async fn head_object(&self, key: &str) -> Result<(), StoreError> {
        // rusty-s3 doesn't ship a HEAD action; fall back to a presigned
        // GET + HTTP HEAD on the same URL.
        let action = GetObject::new(&self.bucket, Some(&self.creds), key);
        let url = action.sign(Duration::from_secs(30));
        let resp = self.http.head(url).send().await?;
        match resp.status().as_u16() {
            200 => Ok(()),
            404 => Err(StoreError::NotFound),
            s => Err(StoreError::Upstream {
                status: s,
                body: format!("HEAD {}", resp.status()),
            }),
        }
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        let action = DeleteObject::new(&self.bucket, Some(&self.creds), key);
        let url = action.sign(Duration::from_secs(60));
        let resp = self.http.delete(url).send().await?;
        let status = resp.status();
        if status.is_success() || status.as_u16() == 404 {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(StoreError::Upstream {
                status: status.as_u16(),
                body,
            })
        }
    }
}
