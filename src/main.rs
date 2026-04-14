use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod concept;
mod searcher;
mod display;

use concept::ConceptManager;
use searcher::SearchEngine;

/// ████████████████████████████████████████
/// █  CLACKS - Vulnerability Taxonomy  █
/// █      CLI Search Interface         █
/// ████████████████████████████████████████
#[derive(Parser)]
#[command(name = "clacks")]
#[command(about = "🔍 Search and navigate vulnerability taxonomies", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to clacks data directory
    #[arg(short, long, default_value = "./.clacks")]
    data_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for vulnerabilities by keyword or matchword
    Search {
        /// Search query (vulnerability name, keyword, or matchword)
        query: String,

        /// Show tree visualization
        #[arg(short, long)]
        tree: bool,

        /// Show full details
        #[arg(short, long)]
        detailed: bool,
    },

    /// Install/update clacks data from repository
    Install {
        /// Repository URL or local path
        #[arg(default_value = "https://github.com/XoanOuteiro/clacks.git")]
        source: String,
    },

    /// List all available vulnerability concepts
    List {
        /// Filter by category path
        #[arg(short, long)]
        category: Option<String>,

        /// Show as tree
        #[arg(short, long)]
        tree: bool,
    },

    /// Show details about a specific concept
    Show {
        /// Concept slug (e.g., injection/server_side/sqli)
        slug: String,

        /// Show tree visualization
        #[arg(short, long)]
        tree: bool,
    },
}

fn main() -> Result<()> {
    display::print_banner();

    let cli = Cli::parse();

    let concept_manager = ConceptManager::new(&cli.data_dir)
        .context("Failed to initialize concept manager")?;

    match cli.command {
        Commands::Search { query, tree, detailed } => {
            display::print_section_header("SEARCH RESULTS");
            
            let search_engine = SearchEngine::new(&concept_manager);
            let results = search_engine.search(&query)?;

            if results.is_empty() {
                display::print_error(&format!("No results found for: {}", query));
                return Ok(());
            }

            display::print_info(&format!("Found {} result(s)", results.len()));
            println!();

            for (idx, concept) in results.iter().enumerate() {
                if tree {
                    display::print_tree_view(concept, &concept_manager)?;
                } else if detailed {
                    display::print_detailed_view(concept, idx + 1)?;
                } else {
                    display::print_compact_view(concept, idx + 1)?;
                }
                
                if idx < results.len() - 1 {
                    display::print_separator();
                }
            }
        }

        Commands::Install { source } => {
            display::print_section_header("INSTALLATION");
            display::print_info(&format!("Installing from: {}", source));
            
            concept_manager.install_from_source(&source)?;
            
            display::print_success("Installation complete!");
        }

        Commands::List { category, tree } => {
            display::print_section_header("CONCEPT LISTING");
            
            let concepts = if let Some(cat) = category {
                concept_manager.list_by_category(&cat)?
            } else {
                concept_manager.list_all()?
            };

            if tree {
                display::print_tree_listing(&concepts, &concept_manager)?;
            } else {
                display::print_list_view(&concepts)?;
            }
        }

        Commands::Show { slug, tree } => {
            display::print_section_header("CONCEPT DETAILS");
            
            let concept = concept_manager.get_by_slug(&slug)
                .context(format!("Concept not found: {}", slug))?;

            if tree {
                display::print_tree_view(&concept, &concept_manager)?;
            } else {
                display::print_full_details(&concept)?;
            }
        }
    }

    display::print_footer();
    Ok(())
}
