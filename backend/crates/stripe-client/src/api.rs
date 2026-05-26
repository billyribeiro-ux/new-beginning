//! `StripeApi` trait + the real `reqwest`-backed `StripeClient`.
//!
//! BACKEND.md §8.1. PR #7 ships the two methods needed for `/v1/checkout`:
//! `get_or_create_customer_for_user` and `create_checkout_session`. Other
//! methods (refund, portal, sub mutations) land with their respective PRs.
//!
//! Every write that mutates Stripe state takes an `idempotency_key`
//! parameter — the helper that derives it lives in `common::auth::idempotency`
//! so refund / portal / sub-mutate calls share the same scheme.

use async_trait::async_trait;
use common::ids::UserId;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use std::time::Duration;
use tracing::instrument;

use crate::domain::{CheckoutSession, CreateCheckoutArgs, CustomerRef, PortalSession};

const STRIPE_BASE_URL: &str = "https://api.stripe.com";
/// API version pinned in code; Config can override via `STRIPE_API_VERSION`.
/// Mirrored to every request as the `Stripe-Version` header.
pub const DEFAULT_API_VERSION: &str = "2024-12-18.acacia";

#[derive(Debug, thiserror::Error)]
pub enum StripeError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("stripe {status}: {code} — {message}")]
    Api {
        status: u16,
        code: String,
        message: String,
    },
    #[error("stripe response was not the expected shape: {0}")]
    BadResponse(String),
}

#[async_trait]
pub trait StripeApi: Send + Sync {
    /// Idempotent on `user_id`: a re-call for the same user returns the
    /// previously-issued customer ref. The idempotency-key strategy
    /// (`common::auth::idempotency`) makes the underlying API call a no-op
    /// on retry.
    async fn get_or_create_customer_for_user(
        &self,
        user_id: UserId,
        email: &str,
        name: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<CustomerRef, StripeError>;

    /// Create a Checkout Session. Mode chosen by the caller (Payment vs.
    /// Subscription). The returned URL is what the BFF redirects to.
    async fn create_checkout_session(
        &self,
        args: CreateCheckoutArgs,
        idempotency_key: Option<&str>,
    ) -> Result<CheckoutSession, StripeError>;

    /// Create a Customer Portal session — Stripe-hosted billing self-serve
    /// (BACKEND.md §8.6). The returned URL is what the BFF redirects to;
    /// `return_url` is where Stripe sends the user back after.
    async fn create_customer_portal_session(
        &self,
        customer: &CustomerRef,
        return_url: &str,
        idempotency_key: Option<&str>,
    ) -> Result<PortalSession, StripeError>;

    /// Fetch an invoice — needed by the worker (PR #11) to resolve the
    /// short-lived `invoice_pdf` URL Stripe hosts. The PDF URL rotates on
    /// some plans; we re-fetch each time we plan to download it, rather
    /// than persisting the URL from the original webhook payload.
    async fn get_invoice(&self, invoice_id: &str) -> Result<StripeInvoice, StripeError>;

    /// Trigger a full refund against a `payment_intent`. Idempotency-Key
    /// MUST be passed — refund is the textbook double-fire-disaster path.
    /// The actual entitlement revocation happens when Stripe POSTs back
    /// `charge.refunded` to the webhook (the api binary doesn't revoke
    /// inline — keeps the source of truth single).
    async fn refund_payment_intent(
        &self,
        payment_intent_id: &str,
        idempotency_key: &str,
    ) -> Result<StripeRefund, StripeError>;
}

#[derive(Debug, Clone, Deserialize)]
pub struct StripeRefund {
    pub id: String,
    pub status: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub charge: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StripeInvoice {
    pub id: String,
    /// Pre-signed PDF download URL hosted by Stripe. Short-TTL; fetch and
    /// re-host into R2 ASAP rather than storing.
    pub invoice_pdf: Option<String>,
    pub hosted_invoice_url: Option<String>,
    pub status: Option<String>,
    pub number: Option<String>,
}

#[derive(Clone)]
pub struct StripeClient {
    http: Client,
    secret_key: SecretString,
    api_version: String,
}

impl StripeClient {
    pub fn new(secret_key: SecretString, api_version: Option<String>) -> Result<Self, StripeError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self {
            http,
            secret_key,
            api_version: api_version.unwrap_or_else(|| DEFAULT_API_VERSION.to_string()),
        })
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.secret_key.expose_secret())
    }

    async fn get_json<R: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<R, StripeError> {
        let url = format!("{STRIPE_BASE_URL}{path}");
        let resp = self
            .http
            .get(&url)
            .header("Authorization", self.auth_header())
            .header("Stripe-Version", &self.api_version)
            .send()
            .await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if status.is_success() {
            serde_json::from_slice::<R>(&bytes).map_err(|e| StripeError::BadResponse(e.to_string()))
        } else {
            let body: StripeApiError =
                serde_json::from_slice(&bytes).unwrap_or_else(|_| StripeApiError {
                    error: StripeApiErrorBody {
                        code: Some("unknown".into()),
                        message: Some(
                            std::str::from_utf8(&bytes)
                                .unwrap_or("<non-utf8>")
                                .to_string(),
                        ),
                    },
                });
            Err(StripeError::Api {
                status: status.as_u16(),
                code: body.error.code.unwrap_or_else(|| "unknown".into()),
                message: body.error.message.unwrap_or_default(),
            })
        }
    }

    async fn post_form<R: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        form: &[(String, String)],
        idempotency_key: Option<&str>,
    ) -> Result<R, StripeError> {
        let url = format!("{STRIPE_BASE_URL}{path}");
        let mut req = self
            .http
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Stripe-Version", &self.api_version)
            .form(form);
        if let Some(k) = idempotency_key {
            req = req.header("Idempotency-Key", k);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if status.is_success() {
            serde_json::from_slice::<R>(&bytes).map_err(|e| StripeError::BadResponse(e.to_string()))
        } else {
            let body: StripeApiError =
                serde_json::from_slice(&bytes).unwrap_or_else(|_| StripeApiError {
                    error: StripeApiErrorBody {
                        code: Some("unknown".into()),
                        message: Some(
                            std::str::from_utf8(&bytes)
                                .unwrap_or("<non-utf8>")
                                .to_string(),
                        ),
                    },
                });
            Err(StripeError::Api {
                status: status.as_u16(),
                code: body.error.code.unwrap_or_else(|| "unknown".into()),
                message: body.error.message.unwrap_or_default(),
            })
        }
    }
}

#[derive(Deserialize)]
struct StripeApiError {
    error: StripeApiErrorBody,
}

#[derive(Deserialize)]
struct StripeApiErrorBody {
    code: Option<String>,
    message: Option<String>,
}

#[derive(Deserialize)]
struct CustomerCreateResponse {
    id: String,
}

#[derive(Deserialize)]
struct CheckoutSessionCreateResponse {
    id: String,
    url: String,
}

#[async_trait]
impl StripeApi for StripeClient {
    #[instrument(skip(self, idempotency_key), fields(user_id = %user_id))]
    async fn get_or_create_customer_for_user(
        &self,
        user_id: UserId,
        email: &str,
        name: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<CustomerRef, StripeError> {
        // Stripe doesn't have a true "get-or-create" — we always POST
        // /v1/customers. The Stripe-Idempotency-Key header ensures a retry
        // returns the original customer instead of creating a duplicate.
        let mut form: Vec<(String, String)> = vec![
            ("email".into(), email.to_string()),
            ("metadata[tradeflex_user_id]".into(), user_id.to_string()),
        ];
        if let Some(n) = name {
            form.push(("name".into(), n.to_string()));
        }
        let resp: CustomerCreateResponse = self
            .post_form("/v1/customers", &form, idempotency_key)
            .await?;
        Ok(CustomerRef { id: resp.id })
    }

    #[instrument(skip(self, idempotency_key), fields(order_id = %args.order_id, mode = ?args.mode))]
    async fn create_checkout_session(
        &self,
        args: CreateCheckoutArgs,
        idempotency_key: Option<&str>,
    ) -> Result<CheckoutSession, StripeError> {
        let mut form: Vec<(String, String)> = vec![
            ("mode".into(), args.mode.as_str().to_string()),
            ("customer".into(), args.customer.id),
            ("success_url".into(), args.success_url),
            ("cancel_url".into(), args.cancel_url),
            ("client_reference_id".into(), args.order_id.to_string()),
            ("metadata[order_id]".into(), args.order_id.to_string()),
            ("metadata[user_id]".into(), args.user_id.to_string()),
        ];
        // Stripe's line_items[N][price]=...&line_items[N][quantity]=...
        for (i, line) in args.lines.iter().enumerate() {
            form.push((format!("line_items[{i}][price]"), line.price_id.clone()));
            form.push((
                format!("line_items[{i}][quantity]"),
                line.quantity.to_string(),
            ));
        }
        let resp: CheckoutSessionCreateResponse = self
            .post_form("/v1/checkout/sessions", &form, idempotency_key)
            .await?;
        Ok(CheckoutSession {
            id: resp.id,
            url: resp.url,
        })
    }

    #[instrument(skip(self, idempotency_key), fields(customer = %customer.id))]
    async fn create_customer_portal_session(
        &self,
        customer: &CustomerRef,
        return_url: &str,
        idempotency_key: Option<&str>,
    ) -> Result<PortalSession, StripeError> {
        let form: Vec<(String, String)> = vec![
            ("customer".into(), customer.id.clone()),
            ("return_url".into(), return_url.to_string()),
        ];
        let resp: PortalSessionCreateResponse = self
            .post_form("/v1/billing_portal/sessions", &form, idempotency_key)
            .await?;
        Ok(PortalSession {
            id: resp.id,
            url: resp.url,
        })
    }

    #[instrument(skip(self))]
    async fn get_invoice(&self, invoice_id: &str) -> Result<StripeInvoice, StripeError> {
        let path = format!("/v1/invoices/{invoice_id}");
        self.get_json::<StripeInvoice>(&path).await
    }

    #[instrument(skip(self, idempotency_key))]
    async fn refund_payment_intent(
        &self,
        payment_intent_id: &str,
        idempotency_key: &str,
    ) -> Result<StripeRefund, StripeError> {
        let form: Vec<(String, String)> = vec![
            ("payment_intent".into(), payment_intent_id.to_string()),
            ("reason".into(), "requested_by_customer".into()),
        ];
        self.post_form::<StripeRefund>("/v1/refunds", &form, Some(idempotency_key))
            .await
    }
}

#[derive(Deserialize)]
struct PortalSessionCreateResponse {
    id: String,
    url: String,
}
