use serde::{Deserialize, Serialize};

const DEFAULT_CHUNK_SIZE: usize = 1_000;
const DEFAULT_CHUNK_OVERLAP: usize = 100;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SourceRef {
    File { path: String },
    Url { uri: String },
    Object { uri: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngestOptions {
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: usize,
    #[serde(default)]
    pub ocr: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngestRequest {
    pub collection: String,
    pub source: SourceRef,
    #[serde(default)]
    pub options: Option<IngestOptions>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentChunk {
    pub chunk_id: String,
    pub content: String,
    pub source_uri: String,
    pub chunk_index: usize,
    pub metadata: serde_json::Value,
}

fn default_chunk_size() -> usize {
    DEFAULT_CHUNK_SIZE
}

fn default_chunk_overlap() -> usize {
    DEFAULT_CHUNK_OVERLAP
}

pub fn chunk_text(
    text: &str,
    source_uri: impl Into<String>,
    metadata: serde_json::Value,
    options: Option<&IngestOptions>,
) -> Result<Vec<DocumentChunk>, String> {
    let chunk_size = options
        .map(|value| value.chunk_size)
        .unwrap_or(DEFAULT_CHUNK_SIZE);

    let overlap = options
        .map(|value| value.chunk_overlap)
        .unwrap_or(DEFAULT_CHUNK_OVERLAP);

    if chunk_size == 0 {
        return Err("chunk_size must be greater than zero".into());
    }

    if overlap >= chunk_size {
        return Err("chunk_overlap must be smaller than chunk_size".into());
    }

    let source_uri = source_uri.into();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while start < words.len() {
        let end = (start + chunk_size).min(words.len());
        let content = words[start..end].join(" ");

        chunks.push(DocumentChunk {
            chunk_id: format!("{source_uri}#{index}"),
            content,
            source_uri: source_uri.clone(),
            chunk_index: index,
            metadata: metadata.clone(),
        });

        if end == words.len() {
            break;
        }

        start = end - overlap;
        index += 1;
    }

    Ok(chunks)
}
