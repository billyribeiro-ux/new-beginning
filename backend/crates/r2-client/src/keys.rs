//! R2 key conventions (BACKEND.md §9.2). Centralized so a typo in one
//! call site can't put an invoice PDF into the wrong prefix.

use common::ids::{InvoiceId, ProductId, UserId};

pub fn invoice_pdf(user_id: UserId, invoice_id: InvoiceId) -> String {
    format!("invoices/{user_id}/{invoice_id}.pdf")
}

pub fn indicator_download(product_id: ProductId, version: &str, filename: &str) -> String {
    format!("downloads/{product_id}/{version}/{filename}")
}

pub fn data_export(user_id: UserId, job_id: &str) -> String {
    format!("exports/{user_id}/{job_id}.json.gz")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_shapes_are_stable() {
        let u = UserId::new();
        let i = InvoiceId::new();
        assert!(invoice_pdf(u, i).starts_with("invoices/"));
        assert!(invoice_pdf(u, i).ends_with(".pdf"));
    }
}
