# Content Ingestion

## Architecture

Resonique ingests external content through a staged pipeline:

```text
Source URI
   ↓
Resolver and security validation
   ↓
Downloader
   ↓
Format extractor
   ↓
Normalizer
   ↓
Chunker
   ↓
Embedding provider
   ↓
WAL and vector segment
   ↓
Content storage and provenance metadata
```

## Supported source types

| Source | Status |
|---|---|
| Local file | Planned |
| HTTP/HTTPS | Planned |
| S3-compatible object storage | Planned |
| Google Cloud Storage | Planned |
| Azure Blob Storage | Planned |
| Direct text input | Initial implementation |

## Supported formats

| Format | Extraction |
|---|---|
| PDF | Text extraction with OCR fallback |
| DOCX | Paragraph and table extraction |
| XLSX | Sheet, row, and cell extraction |
| TXT/Markdown/HTML | Direct extraction |
| Images | OCR |

## Security requirements

- Never allow unrestricted remote access to arbitrary local paths.
- Permit only configured URI schemes and storage locations.
- Prevent SSRF against localhost, private networks, and cloud metadata services.
- Enforce download, decompression, page, sheet, and archive limits.
- Validate MIME type from content, not only the filename.
- Store credentials outside request payloads.
- Apply download timeouts and cancellation.
- Use content hashes for idempotency.
- Store source provenance with every chunk.

## Storage model

The original file should be stored in durable content storage. Each vector
represents a text chunk and contains metadata such as:

```json
{
  "document_id": "sha256:...",
  "source_uri": "s3://bucket/file.pdf",
  "content_uri": "content://documents/...",
  "chunk_index": 0,
  "page": 1,
  "content_hash": "sha256:..."
}
```

The complete document should not be placed inside vector metadata.

## Processing model

Ingestion should run asynchronously as a job. The request returns a job ID,
while the job status reports:

- queued;
- downloading;
- extracting;
- chunking;
- embedding;
- indexing;
- completed;
- failed;
- cancelled.

## Implementation order

1. Add source validation and direct-text ingestion.
2. Add local-file and HTTP download adapters.
3. Add PDF, DOCX, and XLSX extractors.
4. Add OCR as an optional worker dependency.
5. Add embedding-provider abstraction.
6. Add asynchronous ingestion jobs.
7. Add cloud-storage adapters.
8. Add integration, security, and performance tests.