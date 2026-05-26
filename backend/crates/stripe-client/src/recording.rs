//! `RecordingStripeApi` — test-only fake. Captures every call into an
//! in-memory log and returns deterministic responses. Used by integration
//! tests and evidence runs that don't have real Stripe test keys.

use async_trait::async_trait;
use common::ids::UserId;
use std::sync::{Arc, Mutex};

use crate::api::{StripeApi, StripeError, StripeInvoice, StripeRefund};
use crate::domain::{CheckoutSession, CreateCheckoutArgs, CustomerRef, PortalSession};

#[derive(Debug, Clone)]
pub enum Call {
    CreateCustomer {
        user_id: UserId,
        email: String,
        idempotency_key: Option<String>,
    },
    CreateCheckoutSession {
        order_id: String,
        idempotency_key: Option<String>,
    },
    CreatePortalSession {
        customer_id: String,
        return_url: String,
        idempotency_key: Option<String>,
    },
    GetInvoice {
        invoice_id: String,
    },
    RefundPaymentIntent {
        payment_intent_id: String,
        idempotency_key: String,
    },
}

#[derive(Clone, Default)]
pub struct RecordingStripeApi {
    inner: Arc<Mutex<Vec<Call>>>,
}

impl RecordingStripeApi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calls(&self) -> Vec<Call> {
        self.inner.lock().expect("recording mutex").clone()
    }

    pub fn clear(&self) {
        self.inner.lock().expect("recording mutex").clear();
    }
}

#[async_trait]
impl StripeApi for RecordingStripeApi {
    async fn get_or_create_customer_for_user(
        &self,
        user_id: UserId,
        email: &str,
        _name: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> Result<CustomerRef, StripeError> {
        self.inner
            .lock()
            .expect("recording mutex")
            .push(Call::CreateCustomer {
                user_id,
                email: email.to_string(),
                idempotency_key: idempotency_key.map(str::to_owned),
            });
        // Deterministic id derived from the user id so retries collapse.
        Ok(CustomerRef {
            id: format!("cus_fake_{}", user_id.as_uuid().simple()),
        })
    }

    async fn create_checkout_session(
        &self,
        args: CreateCheckoutArgs,
        idempotency_key: Option<&str>,
    ) -> Result<CheckoutSession, StripeError> {
        let order = args.order_id.to_string();
        self.inner
            .lock()
            .expect("recording mutex")
            .push(Call::CreateCheckoutSession {
                order_id: order.clone(),
                idempotency_key: idempotency_key.map(str::to_owned),
            });
        Ok(CheckoutSession {
            id: format!("cs_fake_{}", order),
            url: format!("https://checkout.stripe.test/fake/{order}"),
        })
    }

    async fn create_customer_portal_session(
        &self,
        customer: &CustomerRef,
        return_url: &str,
        idempotency_key: Option<&str>,
    ) -> Result<PortalSession, StripeError> {
        self.inner
            .lock()
            .expect("recording mutex")
            .push(Call::CreatePortalSession {
                customer_id: customer.id.clone(),
                return_url: return_url.to_string(),
                idempotency_key: idempotency_key.map(str::to_owned),
            });
        Ok(PortalSession {
            id: format!("ps_fake_{}", customer.id),
            url: format!("https://billing.stripe.test/fake/{}", customer.id),
        })
    }

    async fn get_invoice(&self, invoice_id: &str) -> Result<StripeInvoice, StripeError> {
        self.inner
            .lock()
            .expect("recording mutex")
            .push(Call::GetInvoice {
                invoice_id: invoice_id.to_string(),
            });
        Ok(StripeInvoice {
            id: invoice_id.to_string(),
            invoice_pdf: Some(format!("https://stripe.test/fake-pdf/{invoice_id}.pdf")),
            hosted_invoice_url: Some(format!("https://stripe.test/fake-hosted/{invoice_id}")),
            status: Some("paid".into()),
            number: Some(format!("INV-FAKE-{invoice_id}")),
        })
    }

    async fn refund_payment_intent(
        &self,
        payment_intent_id: &str,
        idempotency_key: &str,
    ) -> Result<StripeRefund, StripeError> {
        self.inner
            .lock()
            .expect("recording mutex")
            .push(Call::RefundPaymentIntent {
                payment_intent_id: payment_intent_id.to_string(),
                idempotency_key: idempotency_key.to_string(),
            });
        Ok(StripeRefund {
            id: format!("re_fake_{payment_intent_id}"),
            status: Some("succeeded".into()),
            amount: Some(0),
            currency: Some("usd".into()),
            charge: Some(format!("ch_fake_{payment_intent_id}")),
        })
    }
}
