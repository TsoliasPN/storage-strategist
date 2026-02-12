use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::model::{DuplicateFile, DuplicateGroup, DuplicateIntent, DuplicateIntentLabel};

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub disk_mount: Option<String>,
    pub modified: Option<String>,
}

impl FileRecord {
    pub fn from_path(
        path: PathBuf,
        disk_mount: Option<String>,
        modified: Option<String>,
    ) -> Result<Self> {
        let metadata = std::fs::metadata(&path)
            .with_context(|| format!("failed to read metadata for {}", path.display()))?;
        Ok(Self {
            path,
            size_bytes: metadata.len(),
            disk_mount,
            modified,
        })
    }
}

pub fn find_duplicates(
    records: &[FileRecord],
    min_size_bytes: u64,
    warnings: &mut Vec<String>,
) -> Vec<DuplicateGroup> {
    let mut by_size: HashMap<u64, Vec<FileRecord>> = HashMap::new();
    for record in records {
        if record.size_bytes < min_size_bytes {
            continue;
        }
        by_size
            .entry(record.size_bytes)
            .or_default()
            .push(record.clone());
    }

    let mut groups = Vec::new();
    let mut size_keys: Vec<u64> = by_size.keys().copied().collect();
    size_keys.sort_unstable_by(|a, b| b.cmp(a));

    for size in size_keys {
        let candidates = by_size.remove(&size).unwrap_or_default();
        if candidates.len() < 2 {
            continue;
        }

        let mut by_hash: HashMap<String, Vec<FileRecord>> = HashMap::new();
        for candidate in candidates {
            match hash_file(&candidate.path) {
                Ok(hash) => by_hash.entry(hash).or_default().push(candidate),
                Err(err) => warnings.push(format!(
                    "dedupe hash skipped for {}: {}",
                    candidate.path.display(),
                    err
                )),
            }
        }

        for (hash, mut files) in by_hash {
            if files.len() < 2 {
                continue;
            }
            files.sort_by(|a, b| a.path.cmp(&b.path));

            let intent = classify_intent(&files);
            let duplicate_files = files
                .iter()
                .map(|item| DuplicateFile {
                    path: item.path.to_string_lossy().to_string(),
                    disk_mount: item.disk_mount.clone(),
                    modified: item.modified.clone(),
                })
                .collect::<Vec<_>>();

            let wasted = size.saturating_mul((duplicate_files.len() as u64).saturating_sub(1));
            groups.push(DuplicateGroup {
                size_bytes: size,
                hash,
                files: duplicate_files,
                total_wasted_bytes: wasted,
                intent,
            });
        }
    }

    groups.sort_by(|a, b| {
        b.total_wasted_bytes
            .cmp(&a.total_wasted_bytes)
            .then_with(|| b.files.len().cmp(&a.files.len()))
    });
    groups
}

fn hash_file(path: &Path) -> Result<String> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn classify_intent(files: &[FileRecord]) -> DuplicateIntent {
    let backup_keywords = ["backup", "time machine", "history", "mirror", "snapshot"];
    let lowered = files
        .iter()
        .map(|f| f.path.to_string_lossy().to_lowercase())
        .collect::<Vec<_>>();

    if lowered
        .iter()
        .any(|path| backup_keywords.iter().any(|keyword| path.contains(keyword)))
    {
        return DuplicateIntent {
            label: DuplicateIntentLabel::LikelyIntentional,
            rationale:
                "Paths include known backup or snapshot indicators; duplicates may be intentional."
                    .to_string(),
        };
    }

    let unique_mounts = files
        .iter()
        .filter_map(|file| file.disk_mount.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mirrored_suffix = shared_suffix(files, 2);
    if unique_mounts.len() > 1 && mirrored_suffix {
        return DuplicateIntent {
            label: DuplicateIntentLabel::LikelyIntentional,
            rationale:
                "Files share mirrored directory suffixes across multiple mounts; likely sync/backup copies."
                    .to_string(),
        };
    }

    DuplicateIntent {
        label: DuplicateIntentLabel::LikelyRedundant,
        rationale: "No backup or mirror indicators found for this duplicate set.".to_string(),
    }
}

fn shared_suffix(files: &[FileRecord], depth: usize) -> bool {
    if files.len() < 2 {
        return false;
    }

    let mut suffixes = files.iter().map(|file| {
        file.path
            .components()
            .rev()
            .take(depth)
            .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
            .collect::<Vec<_>>()
    });

    let first = match suffixes.next() {
        Some(value) => value,
        None => return false,
    };
    suffixes.all(|suffix| suffix == first)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::{find_duplicates, FileRecord};
    use crate::model::DuplicateIntentLabel;

    #[test]
    fn groups_duplicates_by_size_and_hash() {
        let temp = TempDir::new().expect("tempdir");
        let a = temp.path().join("a.bin");
        let b = temp.path().join("b.bin");
        let c = temp.path().join("c.bin");

        fs::write(&a, b"duplicate-content").expect("write a");
        fs::write(&b, b"duplicate-content").expect("write b");
        fs::write(&c, b"unique-content").expect("write c");

        let records = vec![
            FileRecord::from_path(a, Some("A:/".to_string()), None).expect("record a"),
            FileRecord::from_path(b, Some("B:/".to_string()), None).expect("record b"),
            FileRecord::from_path(c, Some("A:/".to_string()), None).expect("record c"),
        ];

        let mut warnings = Vec::new();
        let groups = find_duplicates(&records, 1, &mut warnings);

        assert!(warnings.is_empty());
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].files.len(), 2);
        assert_eq!(
            groups[0].intent.label,
            DuplicateIntentLabel::LikelyRedundant
        );
    }
}
