use crate::state::CloudState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use cid::Cid;
use multihash::Multihash;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use surrealdb::sql::Thing;

pub const TABLE: &str = "artist";

const SHA2_256_CODE: u64 = 0x12;
const RAW_CODEC: u64 = 0x55;

/// Persisted artist body. The artist's *identity* is the (normalised) name
/// — the SurrealDB id is `CIDv1-raw(sha256(canonical(name)))`. `roles` and
/// `imageUrl` are presentation/aggregation fields that may grow as the
/// artist appears in more firkins; mutating them does **not** roll the id.
///
/// `roles` is a deduped multiset of every role/title the artist has been
/// upserted with — e.g. an actor in five movies (each upsert carrying a
/// distinct `Actor as <character>` role) ends up with all five roles in a
/// single record rather than five duplicate artist records.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtistBody {
    pub name: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(rename = "imageUrl", default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
}

/// Canonical body used for content-addressing — only the (normalised) name
/// participates. Roles and image url are mutable presentation fields.
#[derive(Serialize)]
struct ArtistIdentityView<'a> {
    name: &'a str,
}

/// Full body pinned to IPFS on every persistence event (create + role merge
/// + presentation edit). The IPFS CID of this view changes when roles or
/// imageUrl change; the SurrealDB id (computed from the identity view)
/// stays stable.
#[derive(Serialize)]
struct ArtistFullBodyView<'a> {
    name: &'a str,
    roles: &'a [String],
    #[serde(rename = "imageUrl", skip_serializing_if = "Option::is_none")]
    image_url: Option<&'a str>,
}

/// Lowercase + trim + collapse internal whitespace so "Tom Hanks", "tom
/// hanks", and "Tom  Hanks" all hash to the same id. The persisted `name`
/// field still keeps the first-seen capitalisation; only the hashed
/// representation is normalised.
fn normalise_name(name: &str) -> String {
    name.trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn serialize_full_body(body: &ArtistBody) -> String {
    let view = ArtistFullBodyView {
        name: &body.name,
        roles: &body.roles,
        image_url: body.image_url.as_deref(),
    };
    serde_json::to_string_pretty(&view).expect("ArtistFullBodyView serializes to JSON")
}

pub fn compute_artist_cid(body: &ArtistBody) -> String {
    let normalised = normalise_name(&body.name);
    let view = ArtistIdentityView {
        name: normalised.as_str(),
    };
    let json = serde_json::to_string_pretty(&view).expect("ArtistIdentityView serializes to JSON");
    let digest = Sha256::digest(json.as_bytes());
    let mh = Multihash::<64>::wrap(SHA2_256_CODE, &digest)
        .expect("sha2-256 digest fits in multihash");
    Cid::new_v1(RAW_CODEC, mh).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistDoc {
    pub id: Option<Thing>,
    pub name: String,
    /// All roles/titles the artist has been recorded with. Deduped.
    /// Migrated automatically from the prior single-`role` schema via the
    /// serde alias.
    #[serde(default, alias = "role")]
    pub roles: ArtistRolesField,
    #[serde(rename = "imageUrl", default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Backwards-compat shim: deserializes either the new `roles: Vec<String>`
/// or the old `role: Option<String>` so existing rows load cleanly. Always
/// serializes as the new array form.
#[derive(Debug, Clone, Default)]
pub struct ArtistRolesField(pub Vec<String>);

impl Serialize for ArtistRolesField {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de> Deserialize<'de> for ArtistRolesField {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum AnyRoles {
            Many(Vec<String>),
            One(String),
            None_,
        }
        Ok(match AnyRoles::deserialize(d)? {
            AnyRoles::Many(v) => ArtistRolesField(v),
            AnyRoles::One(s) => {
                if s.trim().is_empty() {
                    ArtistRolesField(Vec::new())
                } else {
                    ArtistRolesField(vec![s])
                }
            }
            AnyRoles::None_ => ArtistRolesField(Vec::new()),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtistDto {
    pub id: String,
    pub name: String,
    pub roles: Vec<String>,
    #[serde(rename = "imageUrl", skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ArtistDoc> for ArtistDto {
    fn from(doc: ArtistDoc) -> Self {
        let id = doc
            .id
            .as_ref()
            .map(|t| t.id.to_raw())
            .unwrap_or_default();
        Self {
            id,
            name: doc.name,
            roles: doc.roles.0,
            image_url: doc.image_url,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        }
    }
}

/// Single-occurrence inbound shape used by catalog/search responses and
/// the firkin create/update flows. Each upsert call carries one optional
/// role; the server merges that role into the canonical `roles` array on
/// the existing record (or seeds a new record).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpsertArtistRequest {
    pub name: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(rename = "imageUrl", default)]
    pub image_url: Option<String>,
}

pub fn router() -> Router<CloudState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", put(update).delete(delete).get(get_one))
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

/// Sanitise + expand `role: Option<String>` into a `roles: Vec<String>` of
/// 0 or 1 element. Returns None if the name is empty.
pub fn sanitize(req: UpsertArtistRequest) -> Option<ArtistBody> {
    let trim_one = |s: Option<String>| s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return None;
    }
    let roles: Vec<String> = trim_one(req.role).into_iter().collect();
    Some(ArtistBody {
        name,
        roles,
        image_url: trim_one(req.image_url),
    })
}

async fn list(
    State(state): State<CloudState>,
) -> Result<Json<Vec<ArtistDto>>, (StatusCode, Json<serde_json::Value>)> {
    let docs: Vec<ArtistDoc> = state
        .db
        .select(TABLE)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    let mut dtos: Vec<ArtistDto> = docs.into_iter().map(Into::into).collect();
    dtos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(dtos))
}

async fn get_one(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<Json<ArtistDto>, (StatusCode, Json<serde_json::Value>)> {
    let doc: Option<ArtistDoc> = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    match doc {
        Some(d) => Ok(Json(d.into())),
        None => Err(err_response(StatusCode::NOT_FOUND, "artist not found")),
    }
}

async fn create(
    State(state): State<CloudState>,
    Json(req): Json<UpsertArtistRequest>,
) -> Result<(StatusCode, Json<ArtistDto>), (StatusCode, Json<serde_json::Value>)> {
    let body = sanitize(req).ok_or_else(|| err_response(StatusCode::BAD_REQUEST, "name is required"))?;
    let dto = upsert(&state, body).await?;
    Ok((StatusCode::OK, Json(dto)))
}

/// Create-or-merge an artist by name. Used by the firkin create/update
/// flows so callers can pass inline `{ name, role, imageUrl }` objects and
/// have the server materialise the artist docs + IPFS pins on its behalf,
/// returning stable CIDs to embed in the firkin. If an artist with the
/// same (normalised) name already exists, the new role is merged into its
/// `roles` array (deduped) and the imageUrl is back-filled when missing;
/// the same CID is returned in either case.
pub async fn upsert(
    state: &CloudState,
    body: ArtistBody,
) -> Result<ArtistDto, (StatusCode, Json<serde_json::Value>)> {
    let new_id = compute_artist_cid(&body);

    let existing: Option<ArtistDoc> = state
        .db
        .select((TABLE, new_id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?;
    if let Some(mut existing) = existing {
        let mut changed = false;
        for role in &body.roles {
            if !existing.roles.0.iter().any(|r| r == role) {
                existing.roles.0.push(role.clone());
                changed = true;
            }
        }
        // Back-fill the image URL when the existing record has none.
        if existing.image_url.is_none() && body.image_url.is_some() {
            existing.image_url = body.image_url.clone();
            changed = true;
        }
        if !changed {
            return Ok(existing.into());
        }
        existing.updated_at = Utc::now();
        existing.id = None;
        let merged_body = ArtistBody {
            name: existing.name.clone(),
            roles: existing.roles.0.clone(),
            image_url: existing.image_url.clone(),
        };
        let updated: Option<ArtistDoc> = state
            .db
            .update((TABLE, new_id.as_str()))
            .content(existing)
            .await
            .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db update failed: {e}")))?;
        let dto: ArtistDto = updated
            .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "artist row vanished during merge"))?
            .into();
        pin_artist_body(state, &new_id, serialize_full_body(&merged_body)).await;
        return Ok(dto);
    }

    let now = Utc::now();
    let record = ArtistDoc {
        id: None,
        name: body.name.clone(),
        roles: ArtistRolesField(body.roles.clone()),
        image_url: body.image_url.clone(),
        created_at: now,
        updated_at: now,
    };
    let body_json = serialize_full_body(&body);

    let created: Option<ArtistDoc> = state
        .db
        .create((TABLE, new_id.as_str()))
        .content(record)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db create failed: {e}")))?;

    let dto: ArtistDto = created
        .ok_or_else(|| err_response(StatusCode::INTERNAL_SERVER_ERROR, "artist was not persisted"))?
        .into();

    pin_artist_body(state, &new_id, body_json).await;
    Ok(dto)
}

/// Replace-in-place edit (same CID). Accepts the full multi-role shape so
/// the editor on `/artist/[ipfs]` can rewrite the roles array.
#[derive(Debug, Deserialize)]
pub struct ReplaceArtistRequest {
    pub name: String,
    #[serde(default)]
    pub roles: Option<Vec<String>>,
    #[serde(rename = "imageUrl", default)]
    pub image_url: Option<String>,
}

async fn update(
    State(state): State<CloudState>,
    Path(id): Path<String>,
    Json(req): Json<ReplaceArtistRequest>,
) -> Result<Json<ArtistDto>, (StatusCode, Json<serde_json::Value>)> {
    let trim_one = |s: Option<String>| s.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(err_response(StatusCode::BAD_REQUEST, "name is required"));
    }
    let roles: Vec<String> = req
        .roles
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .fold(Vec::<String>::new(), |mut acc, s| {
            if !acc.iter().any(|r| r == &s) {
                acc.push(s);
            }
            acc
        });
    let image_url = trim_one(req.image_url);

    let mut current: ArtistDoc = state
        .db
        .select((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db select failed: {e}")))?
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "artist not found"))?;
    current.name = name;
    current.roles = ArtistRolesField(roles);
    current.image_url = image_url;
    current.updated_at = Utc::now();
    current.id = None;

    let merged_body = ArtistBody {
        name: current.name.clone(),
        roles: current.roles.0.clone(),
        image_url: current.image_url.clone(),
    };

    let updated: Option<ArtistDoc> = state
        .db
        .update((TABLE, id.as_str()))
        .content(current)
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db update failed: {e}")))?;

    let dto: ArtistDto = updated
        .ok_or_else(|| err_response(StatusCode::NOT_FOUND, "artist not found"))?
        .into();
    pin_artist_body(&state, &id, serialize_full_body(&merged_body)).await;
    Ok(Json(dto))
}

async fn delete(
    State(state): State<CloudState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let removed: Option<ArtistDoc> = state
        .db
        .delete((TABLE, id.as_str()))
        .await
        .map_err(|e| err_response(StatusCode::INTERNAL_SERVER_ERROR, format!("db delete failed: {e}")))?;
    match removed {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(err_response(StatusCode::NOT_FOUND, "artist not found")),
    }
}

#[cfg(not(target_os = "android"))]
async fn pin_artist_body(state: &CloudState, artist_id: &str, body_json: String) {
    let bytes = body_json.into_bytes();
    let size = bytes.len() as u64;
    let info = match state
        .ipfs_manager
        .add_bytes(format!("artist-{artist_id}.json"), bytes)
        .await
    {
        Ok(info) => info,
        Err(e) => {
            tracing::warn!("[artists] failed to pin body for {artist_id}: {e}");
            return;
        }
    };
    if let Err(e) = crate::ipfs_pins::record_pin(
        state,
        info.cid,
        format!("artist://{artist_id}"),
        "application/json".to_string(),
        size,
    )
    .await
    {
        tracing::warn!("[artists] failed to record pin row for {artist_id}: {e}");
    }
}

#[cfg(target_os = "android")]
async fn pin_artist_body(_state: &CloudState, _artist_id: &str, _body_json: String) {}

/// Fetch artist docs for a list of CIDs in one pass. Missing CIDs are
/// silently dropped so callers (firkin DTO assembly) don't fail on
/// dangling references.
pub async fn fetch_many(
    state: &CloudState,
    ids: &[String],
) -> Result<Vec<ArtistDto>, surrealdb::Error> {
    let mut out: Vec<ArtistDto> = Vec::with_capacity(ids.len());
    for id in ids {
        let doc: Option<ArtistDoc> = state.db.select((TABLE, id.as_str())).await?;
        if let Some(d) = doc {
            out.push(d.into());
        }
    }
    Ok(out)
}
