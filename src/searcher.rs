use crate::concept::{Concept, ConceptManager};
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct SearchEngine<'a> {
    manager: &'a ConceptManager,
    matcher: SkimMatcherV2,
}

impl<'a> SearchEngine<'a> {
    pub fn new(manager: &'a ConceptManager) -> Self {
        Self {
            manager,
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search for concepts by query string
    pub fn search(&self, query: &str) -> Result<Vec<Concept>> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(Concept, i64)> = Vec::new();

        for concept in self.manager.get_concepts() {
            let mut score = 0i64;

            // Exact match on title (highest priority)
            if concept.title.to_lowercase() == query_lower {
                score += 1000;
            }

            // Exact match on slug
            if concept.slug.to_lowercase() == query_lower {
                score += 900;
            }

            // Exact match on ID
            if concept.id.to_lowercase() == query_lower {
                score += 900;
            }

            // Exact match on aliases
            for alias in &concept.aliases {
                if alias.to_lowercase() == query_lower {
                    score += 800;
                }
            }

            // Match on any part of hierarchical slug
            for part in concept.slug.split('/') {
                if part.to_lowercase() == query_lower {
                    score += 700;
                }
            }

            // Fuzzy match on title
            if let Some(fuzzy_score) = self.matcher.fuzzy_match(&concept.title.to_lowercase(), &query_lower) {
                score += fuzzy_score;
            }

            // Fuzzy match on slug parts
            for part in concept.slug.split('/') {
                if let Some(fuzzy_score) = self.matcher.fuzzy_match(&part.to_lowercase(), &query_lower) {
                    score += fuzzy_score / 2;
                }
            }

            // Fuzzy match on aliases
            for alias in &concept.aliases {
                if let Some(fuzzy_score) = self.matcher.fuzzy_match(&alias.to_lowercase(), &query_lower) {
                    score += fuzzy_score;
                }
            }

            // Contains match (lower priority)
            if concept.title.to_lowercase().contains(&query_lower) {
                score += 50;
            }

            if concept.slug.to_lowercase().contains(&query_lower) {
                score += 50;
            }

            // Check taxonomy mappings
            for mapping in concept.get_all_taxonomy_slugs() {
                if mapping.to_lowercase().contains(&query_lower) {
                    score += 40;
                }
            }

            // Add to results if score is positive
            if score > 0 {
                results.push((concept.clone(), score));
            }
        }

        // Sort by score (descending)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        // Return sorted concepts
        Ok(results.into_iter().map(|(c, _)| c).collect())
    }

    /// Search by exact slug match
    pub fn search_exact(&self, slug: &str) -> Option<Concept> {
        self.manager.get_by_slug(slug)
    }

    /// Get related concepts (same parent or similar mappings)
    pub fn get_related(&self, concept: &Concept) -> Vec<Concept> {
        let mut related = Vec::new();

        // Get siblings (same parent)
        if let Some(parent) = concept.parent_slug() {
            related.extend(
                self.manager
                    .list_by_category(&parent)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|c| c.slug != concept.slug),
            );
        }

        // Get concepts with matching taxonomy mappings
        let concept_mappings = concept.get_all_taxonomy_slugs();
        for other in self.manager.get_concepts() {
            if other.slug != concept.slug {
                let other_mappings = other.get_all_taxonomy_slugs();
                // Check if they share any taxonomy mappings
                if concept_mappings.iter().any(|m| other_mappings.contains(m)) {
                    if !related.iter().any(|c: &Concept| c.slug == other.slug) {
                        related.push(other.clone());
                    }
                }
            }
        }

        related
    }
}

