use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

// Required for cast_slice::<f32, u8>()

#[derive(Debug, serde::Deserialize)]
pub struct WalRecord {
    pub vector_id: u64,
    pub payload: Vec<u8>,

    #[serde(default)]
    pub metadata: crate::model::VectorMetadata,
}

pub struct Wal {
    file: std::fs::File,
}

impl Wal {
    pub fn open(path: &Path) -> Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self { file })
    }

    pub fn append(
        &mut self,
        vector_id: u64,
        vector: &[f32],
        metadata: &crate::model::VectorMetadata,
    ) -> Result<()> {
        let bytes = bytemuck::cast_slice::<f32, u8>(vector);

        let record = serde_json::json!({
            "vector_id": vector_id,
            "payload": bytes,
            "metadata": metadata,
        });

        let line = serde_json::to_vec(&record)?;
        self.file.write_all(&line)?;
        self.file.write_all(b"\n")?;

        Ok(())
    }

    pub fn replay(path: &Path) -> Result<Vec<WalRecord>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut records = Vec::new();

        use std::io::BufRead;

        for (line_number, line_result) in reader.lines().enumerate() {
            let line = line_result?;

            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<WalRecord>(&line) {
                Ok(record) => records.push(record),
                Err(error) if line_number == 0 || line.ends_with('}') => {
                    return Err(anyhow::anyhow!(
                        "invalid WAL record at line {}: {}",
                        line_number + 1,
                        error
                    ));
                }
                Err(error) => {
                    tracing::warn!(
                        line = line_number + 1,
                        %error,
                        "ignoring incomplete trailing WAL record"
                    );
                    break;
                }
            }
        }

        Ok(records)
    }
}
