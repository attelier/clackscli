use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Mapping to another taxonomy with confidence and notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyMapping {
    pub id: String,
    pub confidence: String,
    #[serde(default)]
    pub note: Option<String>,
}

/// Represents a vulnerability concept from concept.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    /// Unique identifier (hierarchical slug with taxonomy prefix)
    pub id: String,

    /// Hierarchical slug path (e.g., "clacks/injection/server_side/sqli")
    pub slug: String,

    /// Human-readable title
    pub title: String,

    /// Alternative names and keywords for searching
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Mappings to other taxonomies (CWE, OWASP, VRT, etc.)
    #[serde(default)]
    pub mappings: HashMap<String, Vec<TaxonomyMapping>>,

    /// Whether this is a specifier (starts with _)
    #[serde(skip)]
    pub is_specifier: bool,

    /// File system path to the concept
    #[serde(skip)]
    pub path: PathBuf,
}

impl Concept {
    /// Load a concept from a concept.json file
    pub fn from_file(path: &Path, base_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read concept file: {:?}", path))?;

        let mut concept: Concept = serde_json::from_str(&content)
            .context(format!("Failed to parse concept JSON: {:?}", path))?;

        // Check if it's a specifier (directory name starts with _)
        let parent = path.parent().unwrap();
        concept.is_specifier = parent
            .components()
            .any(|c| c.as_os_str().to_string_lossy().starts_with('_'));

        concept.path = path.to_path_buf();

        Ok(concept)
    }

    /// Get the hierarchy levels from the slug (all parts separated by /)
    pub fn get_hierarchy(&self) -> Vec<String> {
        self.slug
            .split('/')
            .map(|s| s.to_string())
            .collect()
    }

    /// Get parent slug (everything except the last component)
    pub fn parent_slug(&self) -> Option<String> {
        let parts: Vec<&str> = self.slug.rsplitn(2, '/').collect();
        if parts.len() > 1 {
            Some(parts[1].to_string())
        } else {
            None
        }
    }

    /// Get all hierarchical slugs (e.g., for "a/b/c" returns ["a", "a/b", "a/b/c"])
    pub fn generate_hierarchical_slugs(&self) -> Vec<String> {
        let mut slugs = Vec::new();
        let parts: Vec<&str> = self.slug.split('/').collect();

        for i in 1..=parts.len() {
            slugs.push(parts[..i].join("/"));
        }

        slugs
    }

    /// Get all taxonomy slugs from mappings (flatten all mapping IDs)
    pub fn get_all_taxonomy_slugs(&self) -> Vec<String> {
        let mut slugs = Vec::new();
        
        for mappings in self.mappings.values() {
            for mapping in mappings {
                slugs.push(mapping.id.clone());
            }
        }
        
        slugs
    }

    /// Get the taxonomy prefix (e.g., "clacks", "cwe", "owasp")
    pub fn get_taxonomy(&self) -> String {
        self.slug
            .split('/')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Get the last component of the slug
    pub fn get_leaf_name(&self) -> String {
        self.slug
            .split('/')
            .last()
            .unwrap_or(&self.slug)
            .to_string()
    }
}

/// Manages loading and searching concepts
pub struct ConceptManager {
    data_dir: PathBuf,
    concepts: Vec<Concept>,
}

impl ConceptManager {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let crosswalk_dir = data_dir.join("crosswalk");
        
        if !crosswalk_dir.exists() {
            return Err(anyhow::anyhow!(
                "Crosswalk directory not found: {:?}\nRun 'clacks install' first",
                crosswalk_dir
            ));
        }

        let concepts = Self::load_concepts(&crosswalk_dir)?;

        Ok(Self {
            data_dir: data_dir.to_path_buf(),
            concepts,
        })
    }

    /// Load all concept.json files from the crosswalk directory
    fn load_concepts(crosswalk_dir: &Path) -> Result<Vec<Concept>> {
        let mut concepts = Vec::new();

        for entry in WalkDir::new(crosswalk_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_name() == "concept.json" {
                match Concept::from_file(entry.path(), crosswalk_dir) {
                    Ok(concept) => concepts.push(concept),
                    Err(e) => {
                        eprintln!("Warning: Failed to load {:?}: {}", entry.path(), e);
                    }
                }
            }
        }

        Ok(concepts)
    }

    pub fn get_concepts(&self) -> &[Concept] {
        &self.concepts
    }

    pub fn get_by_slug(&self, slug: &str) -> Option<Concept> {
        self.concepts.iter().find(|c| c.slug == slug).cloned()
    }

    pub fn list_all(&self) -> Result<Vec<Concept>> {
        Ok(self.concepts.clone())
    }

    pub fn list_by_category(&self, category: &str) -> Result<Vec<Concept>> {
        Ok(self
            .concepts
            .iter()
            .filter(|c| c.slug.starts_with(category))
            .cloned()
            .collect())
    }

    pub fn install_from_source(&self, source: &str) -> Result<()> {
        use std::process::Command;

        // Create data directory if it doesn't exist
        fs::create_dir_all(&self.data_dir)?;

        if source.starts_with("http") || source.starts_with("git") {
            // Clone from git repository
            let output = Command::new("git")
                .args(["clone", "--depth", "1", source])
                .arg(&self.data_dir)
                .output()
                .context("Failed to execute git clone")?;

            if !output.status.success() {
                return Err(anyhow::anyhow!(
                    "Git clone failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        } else {
            // Copy from local path
            let source_path = Path::new(source);
            if !source_path.exists() {
                return Err(anyhow::anyhow!("Source path does not exist: {}", source));
            }

            // Copy crosswalk directory
            let crosswalk_src = source_path.join("crosswalk");
            let crosswalk_dst = self.data_dir.join("crosswalk");

            Self::copy_dir_recursive(&crosswalk_src, &crosswalk_dst)?;
        }

        Ok(())
    }

    fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                Self::copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    pub fn get_children(&self, slug: &str) -> Vec<&Concept> {
        self.concepts
            .iter()
            .filter(|c| {
                if let Some(parent) = c.parent_slug() {
                    parent == slug
                } else {
                    false
                }
            })
            .collect()
    }

    pub fn get_root_concepts(&self) -> Vec<&Concept> {
        self.concepts
            .iter()
            .filter(|c| !c.slug.contains('/'))
            .collect()
    }
}

