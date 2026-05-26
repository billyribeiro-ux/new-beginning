//! `GET /v1/me` and `PATCH /v1/me`.

use axum::extract::State;
use axum::Json;
use common::deserialize::double_option;
use common::error::AppError;
use serde::Deserialize;
use storage::{UserPatch, UsersError};
use validator::Validate;

use crate::auth::AuthSession;
use crate::routes::me::MePayload;
use crate::state::AppState;

pub async fn get_me(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<MePayload>, AppError> {
    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;
    Ok(Json(MePayload::from_user(&user)))
}

/// All fields optional. The handler converts each `Some(v)` to a `UserPatch`
/// entry; `None` leaves the column untouched. `headline` is a tri-state:
/// `null` clears it, an empty string clears it (interpreted as "remove"),
/// and a non-empty string sets it.
#[derive(Debug, Default, Deserialize, Validate)]
pub struct PatchMeRequest {
    #[validate(length(min = 1, max = 80))]
    pub name: Option<String>,
    /// Tri-state via `double_option`:
    /// - absent              → `None`         (leave column alone)
    /// - `"headline": null`  → `Some(None)`   (clear column)
    /// - `"headline": "..."` → `Some(Some(_))` (set column; empty/whitespace
    ///   string also clears)
    #[serde(default, deserialize_with = "double_option")]
    pub headline: Option<Option<String>>,
    #[validate(length(min = 1, max = 64))]
    pub timezone: Option<String>,
    #[validate(length(min = 2, max = 8))]
    pub language: Option<String>,
}

pub async fn patch_me(
    State(state): State<AppState>,
    session: AuthSession,
    Json(body): Json<PatchMeRequest>,
) -> Result<Json<MePayload>, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(format_validation(&e)))?;

    // Normalize headline: an explicit empty-string clears it; a non-empty
    // string that's > 200 chars is rejected.
    let headline = match body.headline {
        None => None,
        Some(None) => Some(None),
        Some(Some(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Some(None)
            } else if trimmed.len() > 200 {
                return Err(AppError::Validation(
                    "headline must be \u{2264} 200 chars".into(),
                ));
            } else {
                Some(Some(trimmed.to_string()))
            }
        }
    };

    let patch = UserPatch {
        name: body.name.map(|s| s.trim().to_string()),
        headline,
        timezone: body.timezone,
        language: body.language,
    };

    let updated = state
        .users
        .update_profile(session.user_id, patch)
        .await
        .map_err(|e| match e {
            UsersError::NotFound => AppError::NotFound,
            other => AppError::Internal(anyhow::anyhow!(other)),
        })?;
    Ok(Json(MePayload::from_user(&updated)))
}

fn format_validation(e: &validator::ValidationErrors) -> String {
    let mut msgs: Vec<String> = e
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| errs.iter().map(move |er| format!("{field}: {}", er.code)))
        .collect();
    msgs.sort();
    msgs.join(", ")
}
