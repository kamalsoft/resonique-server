use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use crate::storage::segment::{Segment, SegmentIndexEntry};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub vector_id: u64,
    pub score: f32,
    pub metadata: crate::model::VectorMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueryFilter {
    pub tag: Option<String>,
    pub metadata_key: Option<String>,
    pub metadata_val: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Metric {
    Cosine,
    L2,
}

pub fn search_segment(
    segment: &mut Segment,
    query: &[f32],
    top_k: usize,
    metric: Metric,
    filter: Option<QueryFilter>,
) -> Result<Vec<SearchResult>> {
    let mut results = Vec::new();

    let entries = segment.index.clone();

    for entry in &entries {
        let metadata = segment.read_metadata(entry)?;

        if let Some(ref f) = filter {
            if let Some(ref tag) = f.tag
                && !metadata.tags.contains(tag)
            {
                continue;
            }

            if let Some(ref key) = f.metadata_key {
                match (&f.metadata_val, metadata.kv.get(key)) {
                    (Some(expected), Some(actual)) if actual == expected => {}
                    (Some(_), _) => continue,
                    (None, Some(_)) => {}
                    (None, None) => continue,
                }
            }
        }

        let vector = read_vector(&mut segment.file, entry)?;

        let score = match metric {
            Metric::Cosine => cosine_similarity(query, &vector),
            Metric::L2 => l2_distance(query, &vector),
        };

        results.push(SearchResult {
            vector_id: entry.vector_id,
            score,
            metadata,
        });
    }

    match metric {
        Metric::Cosine => results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        Metric::L2 => results.sort_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }

    Ok(results.into_iter().take(top_k).collect())
}

fn read_vector(file: &mut File, entry: &SegmentIndexEntry) -> Result<Vec<f32>> {
    file.seek(SeekFrom::Start(entry.offset))?;

    let mut buf = vec![0u8; entry.length as usize];
    file.read_exact(&mut buf)?;

    // Convert &[u8] → &[f32]
    let floats: &[f32] = bytemuck::cast_slice(&buf);

    Ok(floats.to_vec())
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}
