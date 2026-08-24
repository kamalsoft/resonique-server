use crate::ingestion::{DocumentChunk, IngestOptions, IngestRequest, SourceRef, chunk_text};
use serde_json::json;

#[test]
fn chunks_empty_text_into_no_chunks() {
    let chunks = chunk_text("", "file.txt", json!({}), None).unwrap();

    assert!(chunks.is_empty());
}

#[test]
fn chunks_text_without_options_using_defaults() {
    let chunks = chunk_text("hello world", "file.txt", json!({"kind": "text"}), None).unwrap();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_id, "file.txt#0");
    assert_eq!(chunks[0].content, "hello world");
    assert_eq!(chunks[0].source_uri, "file.txt");
    assert_eq!(chunks[0].chunk_index, 0);
    assert_eq!(chunks[0].metadata, json!({"kind": "text"}));
}

#[test]
fn chunks_exact_boundary_without_extra_chunk() {
    let options = IngestOptions {
        chunk_size: 2,
        chunk_overlap: 0,
        ocr: false,
    };

    let chunks = chunk_text("one two", "source", json!({}), Some(&options)).unwrap();

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "one two");
}

#[test]
fn chunks_with_overlap() {
    let options = IngestOptions {
        chunk_size: 3,
        chunk_overlap: 1,
        ocr: false,
    };

    let chunks = chunk_text(
        "one two three four five",
        "source",
        json!({"page": 1}),
        Some(&options),
    )
    .unwrap();

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].content, "one two three");
    assert_eq!(chunks[1].content, "three four five");
    assert_eq!(chunks[1].chunk_id, "source#1");
    assert_eq!(chunks[1].chunk_index, 1);
}

#[test]
fn normalizes_whitespace() {
    let chunks = chunk_text(" one   two\nthree ", "source", json!({}), None).unwrap();

    assert_eq!(chunks[0].content, "one two three");
}

#[test]
fn rejects_zero_chunk_size() {
    let options = IngestOptions {
        chunk_size: 0,
        chunk_overlap: 0,
        ocr: false,
    };

    assert_eq!(
        chunk_text("text", "source", json!({}), Some(&options)).unwrap_err(),
        "chunk_size must be greater than zero"
    );
}

#[test]
fn rejects_overlap_equal_to_chunk_size() {
    let options = IngestOptions {
        chunk_size: 2,
        chunk_overlap: 2,
        ocr: false,
    };

    assert_eq!(
        chunk_text("text", "source", json!({}), Some(&options)).unwrap_err(),
        "chunk_overlap must be smaller than chunk_size"
    );
}

#[test]
fn rejects_overlap_greater_than_chunk_size() {
    let options = IngestOptions {
        chunk_size: 2,
        chunk_overlap: 3,
        ocr: true,
    };

    assert!(chunk_text("text", "source", json!({}), Some(&options)).is_err());
}

#[test]
fn deserializes_all_source_types() {
    let file: IngestRequest = serde_json::from_value(json!({
        "collection": "docs",
        "source": {"type": "file", "path": "/tmp/file.pdf"}
    }))
    .unwrap();

    assert!(matches!(file.source, SourceRef::File { .. }));

    let url: IngestRequest = serde_json::from_value(json!({
        "collection": "docs",
        "source": {"type": "url", "uri": "https://example.com/file.pdf"}
    }))
    .unwrap();

    assert!(matches!(url.source, SourceRef::Url { .. }));

    let object: IngestRequest = serde_json::from_value(json!({
        "collection": "docs",
        "source": {"type": "object", "uri": "s3://bucket/file.pdf"}
    }))
    .unwrap();

    assert!(matches!(object.source, SourceRef::Object { .. }));
}

#[test]
fn deserializes_default_ingestion_options() {
    let request: IngestRequest = serde_json::from_value(json!({
        "collection": "docs",
        "source": {"type": "file", "path": "file.txt"},
        "options": {}
    }))
    .unwrap();

    let options = request.options.unwrap();

    assert_eq!(options.chunk_size, 1_000);
    assert_eq!(options.chunk_overlap, 100);
    assert!(!options.ocr);
}

#[test]
fn preserves_metadata_in_every_chunk() {
    let options = IngestOptions {
        chunk_size: 1,
        chunk_overlap: 0,
        ocr: false,
    };
    let metadata = json!({"document_id": "abc"});

    let chunks = chunk_text("one two", "source", metadata.clone(), Some(&options)).unwrap();

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].metadata, metadata);
    assert_eq!(chunks[1].metadata, metadata);
}

#[test]
fn document_chunk_is_serializable() {
    let chunk = DocumentChunk {
        chunk_id: "source#0".into(),
        content: "text".into(),
        source_uri: "source".into(),
        chunk_index: 0,
        metadata: json!({}),
    };

    let value = serde_json::to_value(chunk).unwrap();

    assert_eq!(value["chunk_id"], "source#0");
    assert_eq!(value["content"], "text");
}
