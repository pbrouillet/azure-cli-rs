/// Cassette — JSON-serialized HTTP interaction recordings.
///
/// Each cassette is a sequence of request/response pairs stored in
/// `tests/recordings/<module>/<test_name>.json`.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A recorded HTTP request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedRequest {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// A recorded HTTP response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedResponse {
    pub status: u16,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: String,
}

/// A single request→response interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CassetteEntry {
    pub request: RecordedRequest,
    pub response: RecordedResponse,
}

/// A cassette holding all recorded interactions for one test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cassette {
    pub entries: Vec<CassetteEntry>,
}

impl Cassette {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Load a cassette from a JSON file.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Save a cassette to a JSON file (atomic: write to .tmp then rename).
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&tmp, data)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Push a new entry into the cassette.
    pub fn push(&mut self, entry: CassetteEntry) {
        self.entries.push(entry);
    }
}

/// Resolve the cassette file path for a given test.
/// Convention: `tests/recordings/<module>/<test_name>.json`
pub fn cassette_path(module: &str, test_name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("recordings")
        .join(module)
        .join(format!("{test_name}.json"))
}
