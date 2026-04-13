//! OrA Configuration
//!
//! Centralized configuration management with environment variable support.
//! Uses sensible defaults with override capability.

use crate::error::{OraError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration for OrA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // =========================================================================
    // Server Configuration
    // =========================================================================
    /// Server host (default: 127.0.0.1)
    pub host: String,

    /// HTTP server port (default: 8001)
    pub port: u16,

    /// WebSocket port (default: 8001, same as HTTP)
    pub ws_port: u16,

    // =========================================================================
    // Paths
    // =========================================================================
    /// Workspace root directory
    pub workspace_root: PathBuf,

    /// Vault file path
    pub vault_path: PathBuf,

    /// Audit log path
    pub audit_path: PathBuf,

    /// Control-plane SQLite path
    pub control_plane_db_path: PathBuf,

    /// Artifact storage root
    pub artifacts_root: PathBuf,

    /// Constitution YAML path
    pub constitution_path: PathBuf,

    /// Config file path
    pub config_path: Option<PathBuf>,

    // =========================================================================
    // LLM Configuration
    // =========================================================================
    /// Default LLM provider (openai, anthropic, minimax, etc.)
    pub llm_provider: String,

    /// Default model name
    pub default_model: String,

    /// Alias for default_model (used by state.rs)
    #[serde(default)]
    pub llm_model: Option<String>,

    /// API key (can also come from environment)
    #[serde(default)]
    pub llm_api_key: Option<String>,

    /// Base URL for API (can be overridden per provider)
    pub api_base_url: Option<String>,

    /// Alias for api_base_url (used by state.rs)
    #[serde(default)]
    pub llm_base_url: Option<String>,

    /// Maximum tokens in response
    pub max_tokens: u32,

    /// Temperature for generation (0.0 - 2.0)
    pub temperature: f32,

    // =========================================================================
    // Memory / Vector Configuration
    // =========================================================================
    /// Optional Qdrant endpoint for vector-backed memory search
    pub qdrant_url: Option<String>,

    /// Qdrant collection name
    pub qdrant_collection: String,

    // =========================================================================
    // Security Configuration
    // =========================================================================
    /// Maximum authority level without escalation (0-5)
    pub max_authority_level: u8,

    /// Session timeout in seconds (default: 3600)
    pub session_timeout: u64,

    /// Enable security gates (default: true)
    pub security_gates_enabled: bool,

    // =========================================================================
    // Constitution Configuration
    // =========================================================================
    /// Constitution version
    pub constitution_version: String,

    /// Enable constitution enforcement (default: true)
    pub constitution_enabled: bool,

    // =========================================================================
    // Browser Configuration
    // =========================================================================
    /// Enable browser mission support
    pub browser_enabled: bool,

    /// Optional command for an external local browser harness
    pub browser_command: Option<String>,

    // =========================================================================
    // User Configuration
    // =========================================================================
    /// Default user name
    pub user_name: String,

    // =========================================================================
    // Feature Flags
    // =========================================================================
    /// Enable debug logging
    pub debug: bool,

    /// Enable verbose error messages
    pub verbose_errors: bool,
}

impl Default for Config {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let ora_home = home.join(".ora");

        Self {
            // Server
            host: "127.0.0.1".to_string(),
            port: 8001,
            ws_port: 8001,

            // Paths
            workspace_root: home.join("ora-workspace"),
            vault_path: ora_home.join("vault.enc"),
            audit_path: ora_home.join("audit.log"),
            control_plane_db_path: ora_home.join("state").join("control_plane.sqlite"),
            artifacts_root: ora_home.join("artifacts"),
            constitution_path: default_constitution_path(&home),
            config_path: Some(ora_home.join("config.toml")),

            // LLM
            llm_provider: "ollama".to_string(),
            default_model: "auto".to_string(),
            llm_model: None,
            llm_api_key: None,
            api_base_url: Some("http://localhost:11434".to_string()),
            llm_base_url: None,
            max_tokens: 1024,
            temperature: 0.7,

            // Memory / vector
            qdrant_url: None,
            qdrant_collection: "ora_memory".to_string(),

            // Security
            max_authority_level: 3,
            session_timeout: 3600,
            security_gates_enabled: true,

            // Constitution
            constitution_version: "1.0.0".to_string(),
            constitution_enabled: true,

            // Browser
            browser_enabled: true,
            browser_command: None,

            // User
            user_name: "User".to_string(),

            // Features
            debug: false,
            verbose_errors: false,
        }
    }
}

impl Config {
    /// Load configuration from file and environment.
    pub fn load() -> Result<Self> {
        let mut config = Self::default();

        if let Some(ref path) = config.config_path {
            if path.exists() {
                let contents =
                    std::fs::read_to_string(path).map_err(|e| OraError::FileSystemError {
                        path: path.to_string_lossy().to_string(),
                        message: e.to_string(),
                    })?;

                let file_config: FileConfig =
                    toml::from_str(&contents).map_err(|e| OraError::ConfigError {
                        field: "file".to_string(),
                        message: e.to_string(),
                    })?;

                config.apply_file_config(file_config);
            }
        }

        config.apply_env_vars();
        config.ensure_directories()?;

        Ok(config)
    }

    /// Load from a specific config file path.
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        let mut config = Self::default();

        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            let file_config: FileConfig = toml::from_str(&contents)?;
            config.apply_file_config(file_config);
        }

        config.apply_env_vars();
        config.ensure_directories()?;

        Ok(config)
    }

    /// Apply configuration from file.
    fn apply_file_config(&mut self, config: FileConfig) {
        if let Some(server) = config.server {
            self.host = server.host.unwrap_or(self.host.clone());
            self.port = server.port.unwrap_or(self.port);
            self.ws_port = server.ws_port.unwrap_or(self.ws_port);
        }

        if let Some(paths) = config.paths {
            self.workspace_root = paths.workspace_root.unwrap_or(self.workspace_root.clone());
            self.vault_path = paths.vault_path.unwrap_or(self.vault_path.clone());
            self.audit_path = paths.audit_path.unwrap_or(self.audit_path.clone());
            self.control_plane_db_path = paths
                .control_plane_db_path
                .unwrap_or(self.control_plane_db_path.clone());
            self.artifacts_root = paths.artifacts_root.unwrap_or(self.artifacts_root.clone());
            self.constitution_path = paths
                .constitution_path
                .unwrap_or(self.constitution_path.clone());
        }

        if let Some(llm) = config.llm {
            self.llm_provider = llm.provider.unwrap_or(self.llm_provider.clone());
            self.default_model = llm.model.unwrap_or(self.default_model.clone());
            if let Some(api_base_url) = llm.api_base_url {
                self.api_base_url = Some(api_base_url.clone());
                self.llm_base_url = Some(api_base_url);
            }
            self.max_tokens = llm.max_tokens.unwrap_or(self.max_tokens);
            self.temperature = llm.temperature.unwrap_or(self.temperature);
        }

        if let Some(control_plane) = config.control_plane {
            self.qdrant_url = control_plane.qdrant_url.or(self.qdrant_url.clone());
            self.qdrant_collection = control_plane
                .qdrant_collection
                .unwrap_or(self.qdrant_collection.clone());
        }

        if let Some(security) = config.security {
            self.max_authority_level = security
                .max_authority_level
                .unwrap_or(self.max_authority_level);
            self.session_timeout = security.session_timeout.unwrap_or(self.session_timeout);
            self.security_gates_enabled = security
                .security_gates_enabled
                .unwrap_or(self.security_gates_enabled);
        }

        if let Some(constitution) = config.constitution {
            self.constitution_version = constitution
                .version
                .unwrap_or(self.constitution_version.clone());
            self.constitution_enabled = constitution
                .enabled
                .unwrap_or(self.constitution_enabled);
            self.constitution_path = constitution.path.unwrap_or(self.constitution_path.clone());
        }

        if let Some(browser) = config.browser {
            self.browser_enabled = browser.enabled.unwrap_or(self.browser_enabled);
            self.browser_command = browser.command.or(self.browser_command.clone());
        }

        if let Some(user) = config.user {
            self.user_name = user.name.unwrap_or(self.user_name.clone());
        }

        if let Some(debug) = config.debug {
            self.debug = debug;
        }
    }

    /// Apply environment variable overrides.
    fn apply_env_vars(&mut self) {
        // Server
        if let Ok(host) = std::env::var("ORA_HOST") {
            self.host = host;
        }
        if let Ok(port) = std::env::var("ORA_PORT") {
            if let Ok(p) = port.parse() {
                self.port = p;
            }
        }

        // Paths
        if let Ok(path) = std::env::var("ORA_WORKSPACE_ROOT") {
            self.workspace_root = PathBuf::from(path);
        }
        if let Ok(path) = std::env::var("ORA_VAULT_PATH") {
            self.vault_path = PathBuf::from(path);
        }
        if let Ok(path) = std::env::var("ORA_AUDIT_PATH") {
            self.audit_path = PathBuf::from(path);
        }
        if let Ok(path) = std::env::var("ORA_CONTROL_PLANE_DB_PATH") {
            self.control_plane_db_path = PathBuf::from(path);
        }
        if let Ok(path) = std::env::var("ORA_ARTIFACTS_ROOT") {
            self.artifacts_root = PathBuf::from(path);
        }
        if let Ok(path) = std::env::var("ORA_CONSTITUTION_PATH") {
            self.constitution_path = PathBuf::from(path);
        }

        // LLM
        if let Ok(provider) = std::env::var("ORA_LLM_PROVIDER") {
            self.llm_provider = provider;
        }
        if let Ok(model) = std::env::var("ORA_MODEL") {
            self.default_model = model;
        }
        if let Ok(api_key) = std::env::var("ORA_API_KEY") {
            self.llm_api_key = Some(api_key.clone());
            std::env::set_var("ORA_API_KEY", api_key);
        }
        if let Ok(api_key) = std::env::var("ORA_LLM_API_KEY") {
            self.llm_api_key = Some(api_key);
        }
        if let Ok(base_url) = std::env::var("ORA_API_BASE_URL") {
            self.api_base_url = Some(base_url.clone());
            self.llm_base_url = Some(base_url);
        }
        if let Ok(base_url) = std::env::var("ORA_LLM_BASE_URL") {
            self.llm_base_url = Some(base_url);
        }

        // Memory / vector
        if let Ok(url) = std::env::var("ORA_QDRANT_URL") {
            self.qdrant_url = Some(url);
        }
        if let Ok(collection) = std::env::var("ORA_QDRANT_COLLECTION") {
            self.qdrant_collection = collection;
        }

        // Security
        if let Ok(level) = std::env::var("ORA_MAX_AUTHORITY") {
            if let Ok(l) = level.parse() {
                self.max_authority_level = l;
            }
        }

        // Constitution
        if let Ok(version) = std::env::var("ORA_CONSTITUTION_VERSION") {
            self.constitution_version = version;
        }
        if let Ok(enabled) = std::env::var("ORA_CONSTITUTION_ENABLED") {
            self.constitution_enabled = enabled.to_lowercase() == "true";
        }

        // Browser
        if let Ok(enabled) = std::env::var("ORA_BROWSER_ENABLED") {
            self.browser_enabled = enabled.to_lowercase() == "true";
        }
        if let Ok(command) = std::env::var("ORA_BROWSER_COMMAND") {
            self.browser_command = Some(command);
        }

        // Debug
        if let Ok(debug) = std::env::var("ORA_DEBUG") {
            self.debug = debug.to_lowercase() == "true";
        }
    }

    /// Ensure required directories exist.
    fn ensure_directories(&self) -> Result<()> {
        for path in [
            self.vault_path.parent(),
            self.audit_path.parent(),
            self.control_plane_db_path.parent(),
            Some(self.artifacts_root.as_path()),
        ]
        .into_iter()
        .flatten()
        {
            std::fs::create_dir_all(path)?;
        }

        std::fs::create_dir_all(&self.workspace_root)?;
        Ok(())
    }

    /// Get the API base URL for the configured provider.
    pub fn get_api_base_url(&self) -> String {
        if let Some(ref url) = self.api_base_url {
            return url.clone();
        }

        match self.llm_provider.as_str() {
            "openai" => "https://api.openai.com/v1".to_string(),
            "anthropic" => "https://api.anthropic.com".to_string(),
            "minimax" => "https://api.minimax.chat/v1".to_string(),
            "deepseek" => "https://api.deepseek.com/v1".to_string(),
            "glm" => "https://open.bigmodel.cn/api/paas/v4".to_string(),
            "ollama" | "local" => "http://localhost:11434".to_string(),
            _ => "https://api.openai.com/v1".to_string(),
        }
    }
}

// =========================================================================
// File Configuration Structures (for TOML parsing)
// =========================================================================

#[derive(Debug, Deserialize, Default)]
struct FileConfig {
    #[serde(default)]
    server: Option<ServerConfig>,
    #[serde(default)]
    paths: Option<PathsConfig>,
    #[serde(default)]
    llm: Option<LlmConfig>,
    #[serde(default)]
    control_plane: Option<ControlPlaneConfig>,
    #[serde(default)]
    security: Option<SecurityConfig>,
    #[serde(default)]
    constitution: Option<ConstitutionConfig>,
    #[serde(default)]
    browser: Option<BrowserConfig>,
    #[serde(default)]
    user: Option<UserConfig>,
    #[serde(default)]
    debug: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    #[serde(default)]
    host: Option<String>,
    #[serde(default)]
    port: Option<u16>,
    #[serde(default)]
    ws_port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct PathsConfig {
    #[serde(default)]
    workspace_root: Option<PathBuf>,
    #[serde(default)]
    vault_path: Option<PathBuf>,
    #[serde(default)]
    audit_path: Option<PathBuf>,
    #[serde(default)]
    control_plane_db_path: Option<PathBuf>,
    #[serde(default)]
    artifacts_root: Option<PathBuf>,
    #[serde(default)]
    constitution_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LlmConfig {
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    api_base_url: Option<String>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct ControlPlaneConfig {
    #[serde(default)]
    qdrant_url: Option<String>,
    #[serde(default)]
    qdrant_collection: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecurityConfig {
    #[serde(default)]
    max_authority_level: Option<u8>,
    #[serde(default)]
    session_timeout: Option<u64>,
    #[serde(default)]
    security_gates_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ConstitutionConfig {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct BrowserConfig {
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    command: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserConfig {
    #[serde(default)]
    name: Option<String>,
}

fn default_constitution_path(home: &Path) -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| home.to_path_buf());
    let candidates = [
        current_dir.join("config").join("odin-constitution.yaml"),
        current_dir.join("../config").join("odin-constitution.yaml"),
        home.join(".ora").join("odin-constitution.yaml"),
    ];

    candidates
        .into_iter()
        .find(|candidate| candidate.exists())
        .unwrap_or_else(|| home.join(".ora").join("odin-constitution.yaml"))
}

// =========================================================================
// Helper for getting home directory
// =========================================================================

mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("USERPROFILE").ok().map(PathBuf::from)
        }
        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("HOME").ok().map(PathBuf::from)
        }
    }
}
