use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentHeader {
    pub magic: u32,
    pub version: u32,
    pub entry_count: u64,
    pub index_offset: u64,
    pub payload_offset: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct SegmentIndexEntry {
    pub vector_id: u64,
    pub offset: u64,
    pub length: u32,
    pub metadata_offset: u64,
}

#[derive(Debug)]
pub struct Segment {
    pub header: SegmentHeader,
    pub index: Vec<SegmentIndexEntry>,
    pub file: File,
}

impl Segment {
    pub fn create(path: &Path) -> Result<Self> {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        let header = SegmentHeader {
            magic: 0xDEADBEEF,
            version: 1,
            entry_count: 0,
            index_offset: 0,
            payload_offset: 0,
        };

        let header_bytes = serde_json::to_vec(&header)?;
        file.write_all(&header_bytes)?;

        Ok(Self {
            header,
            index: Vec::new(),
            file,
        })
    }

    pub fn open(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Self::create(path);
        }

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;

        // Read first 1024 bytes to parse header
        let mut buf = vec![0u8; 1024];
        file.seek(SeekFrom::Start(0))?;
        let _ = std::io::Read::read(&mut file, &mut buf)?;

        let end_idx = buf
            .iter()
            .position(|&b| b == b'}')
            .ok_or_else(|| anyhow::anyhow!("Invalid segment header in {:?}", path))?;
        let header_slice = &buf[..=end_idx];
        let mut header: SegmentHeader = serde_json::from_slice(header_slice)?;

        header.payload_offset = (end_idx + 1) as u64;

        Ok(Self {
            header,
            index: Vec::new(),
            file,
        })
    }

    pub fn insert(
        &mut self,
        vector_id: u64,
        vector: &[f32],
        metadata: &crate::model::VectorMetadata,
    ) -> Result<()> {
        let bytes = bytemuck::cast_slice::<f32, u8>(vector);

        let metadata_bytes = serde_json::to_vec(metadata)?;

        let offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(bytes)?;

        let metadata_offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&metadata_bytes)?;

        let entry = SegmentIndexEntry {
            vector_id,
            offset,
            length: bytes.len() as u32,
            metadata_offset,
        };

        self.index.push(entry);
        self.header.entry_count += 1;

        Ok(())
    }

    pub fn read_metadata(
        &mut self,
        entry: &SegmentIndexEntry,
    ) -> Result<crate::model::VectorMetadata> {
        self.file.seek(SeekFrom::Start(entry.metadata_offset))?;
        // Read until the end or we can just read the rest of file/deserialize from JSON.
        // Since we write JSON followed by possibly another record, we can use serde_json::Deserializer
        let mut de = serde_json::Deserializer::from_reader(&self.file);
        let metadata = crate::model::VectorMetadata::deserialize(&mut de)?;
        Ok(metadata)
    }
}
