//! Typed UUID newtypes for primary keys.
//!
//! BACKEND.md §1.6: every PK is generated server-side with `Uuid::now_v7()`.
//! Typed wrappers prevent passing a `UserId` where a `SessionId` is expected,
//! a class of bug that bare `Uuid` cannot prevent.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

macro_rules! typed_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            #[inline]
            pub fn new() -> Self {
                Self(Uuid::now_v7())
            }

            #[inline]
            pub fn from_uuid(u: Uuid) -> Self {
                Self(u)
            }

            #[inline]
            pub fn as_uuid(&self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<Uuid> for $name {
            fn from(u: Uuid) -> Self {
                Self(u)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Uuid {
                id.0
            }
        }
    };
}

typed_id!(
    /// Primary key on `users`.
    UserId
);
typed_id!(
    /// Primary key on `sessions`.
    SessionId
);
typed_id!(
    /// Primary key on `orders`.
    OrderId
);
typed_id!(
    /// Primary key on `products`.
    ProductId
);
typed_id!(
    /// Primary key on `subscription_plans`.
    PlanId
);
typed_id!(
    /// Primary key on `subscriptions`.
    SubscriptionId
);
typed_id!(
    /// Primary key on `invoices`.
    InvoiceId
);
typed_id!(
    /// Primary key on `licenses`.
    LicenseId
);
typed_id!(
    /// Primary key on `enrollments`.
    EnrollmentId
);
typed_id!(
    /// Primary key on `notifications`.
    NotificationId
);
typed_id!(
    /// Primary key on `leads`.
    LeadId
);
typed_id!(
    /// Primary key on `contact_messages`.
    ContactMessageId
);
typed_id!(
    /// Primary key on `audit_log`.
    AuditId
);
typed_id!(
    /// Primary key on `background_jobs`.
    JobId
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_v7_so_ids_are_time_sortable() {
        let a = UserId::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = UserId::new();
        assert!(a < b, "{a} should be less than {b}");
    }

    #[test]
    fn typed_ids_are_distinct_types_at_compile_time() {
        let u = UserId::new();
        let _uuid: Uuid = u.into();
        // The check below is a compile-time guarantee, asserted here by
        // explicit type annotation: passing a UserId to a SessionId-typed
        // slot would fail to compile.
        let s: SessionId = SessionId::new();
        assert_ne!(u.as_uuid(), s.as_uuid());
    }
}
