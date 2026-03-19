#[cfg(not(target_os = "android"))]
pub mod p2p_stream;
pub mod signaling;
pub mod signaling_deploy;
pub mod tmdb;
#[cfg(not(target_os = "android"))]
pub mod torrent;
pub mod jackett;
pub mod torrent_search;
pub mod youtube_meta;
#[cfg(not(target_os = "android"))]
pub mod ytdl;

use crate::AppState;
use serde::Serialize;
use std::collections::HashMap;

/// Describes a module's capabilities for status reporting.
#[derive(Debug, Clone, Serialize)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<ModuleCompatibility>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<ModuleSettingDef>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub link_sources: Vec<ModuleLinkSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_sql: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleCompatibility {
    pub mobile: bool,
    pub computer: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleSettingDef {
    pub key: String,
    pub default: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleLinkSource {
    pub service: String,
    pub label: String,
    pub media_type_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<String>,
}

/// A running process managed by a module.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessStatus {
    pub id: String,
    pub available: bool,
    pub port: u16,
    pub url: String,
    #[serde(rename = "logPrefix")]
    pub log_prefix: String,
}

/// Status information returned by the /api/plugins endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct ModuleStatus {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<ModuleCompatibility>,
    pub processes: Vec<ProcessStatus>,
    pub settings: Vec<ModuleSettingStatus>,
    #[serde(rename = "scheduledTasks")]
    pub scheduled_tasks: Vec<String>,
    #[serde(rename = "schemaTables")]
    pub schema_tables: Vec<SchemaTable>,
    #[serde(rename = "linkSources")]
    pub link_sources: Vec<ModuleLinkSource>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleSettingStatus {
    pub key: String,
    pub value: String,
    pub default: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchemaTable {
    pub name: String,
    pub columns: Vec<String>,
}

/// Trait implemented by each module. Modules register themselves with the ModuleRegistry.
#[allow(unused_variables)]
pub trait Module: Send + Sync {
    fn manifest(&self) -> ModuleManifest;

    /// Return running processes managed by this module.
    fn processes(&self, state: &AppState) -> Vec<ProcessStatus> {
        Vec::new()
    }

    /// Called during initialization after schema and settings are applied.
    fn initialize(&self, state: &AppState) -> Result<(), String> {
        Ok(())
    }

    /// Called during shutdown in reverse registration order.
    fn shutdown(&self) {}
}

/// Registry that manages module lifecycle.
pub struct ModuleRegistry {
    modules: Vec<Box<dyn Module>>,
    initialized: bool,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
            initialized: false,
        }
    }

    /// Register a module. Must be called before initialize().
    pub fn register(&mut self, module: Box<dyn Module>) {
        if self.initialized {
            tracing::warn!(
                "Cannot register module after initialization: {}",
                module.manifest().name
            );
            return;
        }
        self.modules.push(module);
    }

    /// Initialize all registered modules: apply schemas, seed settings, register link sources.
    pub fn initialize(&mut self, state: &AppState) {
        if self.initialized {
            tracing::warn!("ModuleRegistry already initialized");
            return;
        }

        for module in &self.modules {
            let manifest = module.manifest();
            let log_prefix = format!("[module:{}]", manifest.name);

            // 1. Apply schema SQL (module schemas already applied by db init, but allow dynamic ones)
            if let Some(sql) = &manifest.schema_sql {
                let conn = state.db.lock();
                if let Err(e) = conn.execute_batch(sql) {
                    tracing::error!("{} Schema failed: {}", log_prefix, e);
                }
            }

            // 2. Seed settings defaults
            if !manifest.settings.is_empty() {
                let mut entries: HashMap<String, String> = HashMap::new();
                for setting in &manifest.settings {
                    let env_val = setting
                        .env_key
                        .as_ref()
                        .and_then(|k| std::env::var(k).ok());
                    let current = state.settings.get(&setting.key);
                    if current.is_none() {
                        // Setting doesn't exist yet — seed with env value or default
                        entries.insert(
                            setting.key.clone(),
                            env_val.unwrap_or_else(|| setting.default.clone()),
                        );
                    } else if let Some(env_val) = &env_val {
                        // Setting exists but env var is set — override if DB value
                        // is empty or matches the default (i.e. not user-configured)
                        let db_val = current.unwrap();
                        if db_val.is_empty() || db_val == setting.default {
                            entries.insert(setting.key.clone(), env_val.clone());
                        }
                    }
                }
                if !entries.is_empty() {
                    state.settings.set_many(&entries);
                }
            }

            // 3. Register link sources
            for ls in &manifest.link_sources {
                let row = crate::db::repo::link_source::LinkSourceRow {
                    id: uuid::Uuid::new_v4().to_string(),
                    plugin: manifest.name.clone(),
                    service: ls.service.clone(),
                    label: ls.label.clone(),
                    media_type_id: ls.media_type_id.clone(),
                    category_id: ls.category_id.clone(),
                };
                state.link_sources.upsert(&row);
            }

            // 4. Call module-specific initialize
            if let Err(e) = module.initialize(state) {
                tracing::error!("{} Initialize failed: {}", log_prefix, e);
            }

            tracing::info!("{} Initialized (v{})", log_prefix, manifest.version);
        }

        self.initialized = true;
    }

    /// Get status for all registered modules.
    pub fn get_status(&self, state: &AppState) -> Vec<ModuleStatus> {
        self.modules
            .iter()
            .map(|module| {
                let manifest = module.manifest();

                let settings = manifest
                    .settings
                    .iter()
                    .map(|s| ModuleSettingStatus {
                        key: s.key.clone(),
                        value: state
                            .settings
                            .get(&s.key)
                            .unwrap_or_else(|| s.default.clone()),
                        default: s.default.clone(),
                    })
                    .collect();

                let schema_tables = manifest
                    .schema_sql
                    .as_ref()
                    .map(|sql| parse_schema_tables(sql))
                    .unwrap_or_default();

                let processes = module.processes(state);

                ModuleStatus {
                    name: manifest.name.clone(),
                    version: manifest.version,
                    description: manifest.description,
                    source: manifest.source.unwrap_or_else(|| "module".to_string()),
                    compatibility: manifest.compatibility,
                    processes,
                    settings,
                    scheduled_tasks: Vec::new(),
                    schema_tables,
                    link_sources: manifest.link_sources,
                }
            })
            .collect()
    }

    /// Shutdown all modules in reverse order.
    pub fn shutdown(&self) {
        for module in self.modules.iter().rev() {
            let name = module.manifest().name;
            module.shutdown();
            tracing::info!("[module:{}] Shutdown", name);
        }
    }

    /// Update a setting for a specific module.
    pub fn update_setting(
        &self,
        state: &AppState,
        module_name: &str,
        key: &str,
        value: &str,
    ) -> bool {
        let module = self.modules.iter().find(|m| m.manifest().name == module_name);
        if let Some(module) = module {
            let manifest = module.manifest();
            let valid_keys: Vec<&str> = manifest.settings.iter().map(|s| s.key.as_str()).collect();
            if valid_keys.contains(&key) {
                state.settings.set(key, value);
                return true;
            }
        }
        false
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse CREATE TABLE statements from SQL to extract table names and columns.
fn parse_schema_tables(sql: &str) -> Vec<SchemaTable> {
    let mut tables = Vec::new();
    // Simple regex-free parser: look for CREATE TABLE ... ( ... )
    let upper = sql.to_uppercase();
    let mut search_from = 0;

    while let Some(ct_pos) = upper[search_from..].find("CREATE TABLE") {
        let abs_pos = search_from + ct_pos;
        let after_ct = &sql[abs_pos..];

        // Find the opening paren
        if let Some(paren_start) = after_ct.find('(') {
            // Extract table name (between CREATE TABLE [IF NOT EXISTS] and '(')
            let name_part = after_ct[..paren_start].trim();
            let table_name = name_part
                .rsplit_once(char::is_whitespace)
                .map(|(_, name)| name.trim().trim_matches('"'))
                .unwrap_or(name_part.trim_matches('"'));

            // Find matching closing paren
            let mut depth = 0;
            let mut paren_end = None;
            for (i, c) in after_ct[paren_start..].char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            paren_end = Some(paren_start + i);
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if let Some(end) = paren_end {
                let columns_str = &after_ct[paren_start + 1..end];
                let columns: Vec<String> = columns_str
                    .split(',')
                    .filter_map(|col| {
                        let trimmed = col.trim();
                        let first_word = trimmed.split_whitespace().next()?;
                        let upper_word = first_word.to_uppercase();
                        // Skip constraints
                        if matches!(
                            upper_word.as_str(),
                            "PRIMARY" | "FOREIGN" | "UNIQUE" | "CHECK" | "CONSTRAINT"
                        ) {
                            return None;
                        }
                        Some(first_word.trim_matches('"').to_string())
                    })
                    .collect();

                tables.push(SchemaTable {
                    name: table_name.to_string(),
                    columns,
                });

                search_from = abs_pos + end;
            } else {
                search_from = abs_pos + paren_start + 1;
            }
        } else {
            break;
        }
    }

    tables
}
