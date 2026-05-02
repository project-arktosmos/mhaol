use crate::state::CloudState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use mhaol_identity::eip191_recover;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

pub const TABLE: &str = "user";

/// Auth message timestamps must be within this window to be accepted, in
/// either direction. Five minutes is generous enough for slow networks and
/// modest clock skew while still narrowing replay attempts to a small window.
const AUTH_FRESHNESS_SECS: i64 = 300;

const USERNAME_MIN: usize = 1;
const USERNAME_MAX: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<Thing>,
    pub address: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub address: String,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            address: u.address,
            username: u.username,
            created_at: u.created_at,
            updated_at: u.updated_at,
            last_login_at: u.last_login_at,
        }
    }
}

impl IntoResponse for UserDto {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub address: String,
    pub username: String,
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub address: String,
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUsernameRequest {
    pub username: String,
    pub message: String,
    pub signature: String,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list))
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/{address}", get(get_one).put(update_username))
}

fn err_response(
    status: StatusCode,
    message: impl Into<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": message.into() })),
    )
}

/// Normalize an EVM-style address to lowercase 0x-prefixed hex. Returns
/// `None` if the input is not a syntactically valid 20-byte hex address.
fn normalize_address(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let lower = trimmed.to_lowercase();
    let body = lower.strip_prefix("0x").unwrap_or(&lower);
    if body.len() != 40 || !body.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(format!("0x{}", body))
}

fn is_valid_username(name: &str) -> bool {
    let len = name.len();
    if !(USERNAME_MIN..=USERNAME_MAX).contains(&len) {
        return false;
    }
    name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// Verify an EIP-191 signed auth message:
/// - Recovered address matches `expected_address`.
/// - Message is `Mhaol Cloud auth at <RFC3339 timestamp>` and the timestamp
///   sits within `AUTH_FRESHNESS_SECS` of now (in either direction).
fn verify_auth(
    expected_address: &str,
    message: &str,
    signature: &str,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let recovered = eip191_recover(message, signature)
        .map_err(|e| err_response(StatusCode::BAD_REQUEST, format!("invalid signature: {e}")))?;
    if recovered.to_lowercase() != expected_address.to_lowercase() {
        return Err(err_response(
            StatusCode::UNAUTHORIZED,
            "signature does not match address",
        ));
    }

    let ts_str = message
        .strip_prefix("Mhaol Cloud auth at ")
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "message format is invalid"))?;
    let ts = DateTime::parse_from_rfc3339(ts_str)
        .map_err(|e| err_response(StatusCode::BAD_REQUEST, format!("invalid timestamp: {e}")))?
        .with_timezone(&Utc);
    let drift = (Utc::now() - ts).num_seconds().abs();
    if drift > AUTH_FRESHNESS_SECS {
        return Err(err_response(
            StatusCode::UNAUTHORIZED,
            "signed message is stale",
        ));
    }
    Ok(())
}

async fn list(
    State(state): State<CloudState>,
) -> Result<Json<Vec<UserDto>>, (StatusCode, Json<serde_json::Value>)> {
    let users: Vec<User> = state.db.select(TABLE).await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db select failed: {e}"),
        )
    })?;
    let mut dtos: Vec<UserDto> = users.into_iter().map(Into::into).collect();
    dtos.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(Json(dtos))
}

async fn get_one(
    State(state): State<CloudState>,
    Path(address): Path<String>,
) -> Result<Json<UserDto>, (StatusCode, Json<serde_json::Value>)> {
    let normalized = normalize_address(&address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    let user: Option<User> = state
        .db
        .select((TABLE, normalized.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;
    match user {
        Some(u) => Ok(Json(u.into())),
        None => Err(err_response(StatusCode::NOT_FOUND, "user not found")),
    }
}

async fn register(
    State(state): State<CloudState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserDto>), (StatusCode, Json<serde_json::Value>)> {
    let address = normalize_address(&req.address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    let username = req.username.trim().to_string();
    if !is_valid_username(&username) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "username must be 1-32 chars of [A-Za-z0-9-]",
        ));
    }
    verify_auth(&address, &req.message, &req.signature)?;

    let existing: Option<User> = state
        .db
        .select((TABLE, address.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;
    if existing.is_some() {
        return Err(err_response(
            StatusCode::CONFLICT,
            "user already registered for this address",
        ));
    }

    if username_taken(&state, &username, None).await? {
        return Err(err_response(
            StatusCode::CONFLICT,
            "username already taken",
        ));
    }

    let now = Utc::now();
    let record = User {
        id: None,
        address: address.clone(),
        username,
        created_at: now,
        updated_at: now,
        last_login_at: Some(now),
    };
    let created: Option<User> = state
        .db
        .create((TABLE, address.as_str()))
        .content(record)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db create failed: {e}"),
            )
        })?;
    let dto: UserDto = created
        .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "user was not persisted"))?
        .into();
    Ok((StatusCode::CREATED, Json(dto)))
}

async fn login(
    State(state): State<CloudState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<UserDto>, (StatusCode, Json<serde_json::Value>)> {
    let address = normalize_address(&req.address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    verify_auth(&address, &req.message, &req.signature)?;

    let existing: Option<User> = state
        .db
        .select((TABLE, address.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;
    let mut current = existing
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "user not registered"))?;
    current.id = None;
    current.last_login_at = Some(Utc::now());
    current.updated_at = Utc::now();

    let updated: Option<User> = state
        .db
        .update((TABLE, address.as_str()))
        .content(current)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db update failed: {e}"),
            )
        })?;
    let dto: UserDto = updated
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "user not found"))?
        .into();
    Ok(Json(dto))
}

async fn update_username(
    State(state): State<CloudState>,
    Path(address): Path<String>,
    Json(req): Json<UpdateUsernameRequest>,
) -> Result<Json<UserDto>, (StatusCode, Json<serde_json::Value>)> {
    let address = normalize_address(&address)
        .ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "invalid address"))?;
    let username = req.username.trim().to_string();
    if !is_valid_username(&username) {
        return Err(err_response(
            StatusCode::BAD_REQUEST,
            "username must be 1-32 chars of [A-Za-z0-9-]",
        ));
    }
    verify_auth(&address, &req.message, &req.signature)?;

    let existing: Option<User> = state
        .db
        .select((TABLE, address.as_str()))
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db select failed: {e}"),
            )
        })?;
    let mut current = existing
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "user not found"))?;

    if username_taken(&state, &username, Some(&address)).await? {
        return Err(err_response(
            StatusCode::CONFLICT,
            "username already taken",
        ));
    }

    current.id = None;
    current.username = username;
    current.updated_at = Utc::now();

    let updated: Option<User> = state
        .db
        .update((TABLE, address.as_str()))
        .content(current)
        .await
        .map_err(|e| {
            err_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("db update failed: {e}"),
            )
        })?;
    let dto: UserDto = updated
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "user not found"))?
        .into();
    Ok(Json(dto))
}

async fn username_taken(
    state: &CloudState,
    username: &str,
    skip_address: Option<&str>,
) -> Result<bool, (StatusCode, Json<serde_json::Value>)> {
    let users: Vec<User> = state.db.select(TABLE).await.map_err(|e| {
        err_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("db select failed: {e}"),
        )
    })?;
    let lower = username.to_lowercase();
    Ok(users.iter().any(|u| {
        u.username.to_lowercase() == lower
            && skip_address
                .map(|skip| u.address.to_lowercase() != skip.to_lowercase())
                .unwrap_or(true)
    }))
}
