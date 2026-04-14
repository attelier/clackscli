use crate::concept::{Concept, ConceptManager};
use anyhow::Result;
use colored::*;

const CRT_GREEN: Color = Color::TrueColor { r: 0, g: 255, b: 65 };
const CRT_AMBER: Color = Color::TrueColor { r: 255, g: 176, b: 0 };
const CRT_RED: Color = Color::TrueColor { r: 255, g: 50, b: 50 };
const CRT_BLUE: Color = Color::TrueColor { r: 0, g: 200, b: 255 };
const CRT_DIM: Color = Color::TrueColor { r: 100, g: 100, b: 100 };

/// Print the CRT-style banner
pub fn print_banner() {
    let banner = r#"
╔═══════════════════════════════════════════════════════════════════════╗
║                                                                       ║
║   ██████╗██╗      █████╗  ██████╗██╗  ██╗███████╗                     ║
║  ██╔════╝██║     ██╔══██╗██╔════╝██║ ██╔╝██╔════╝                     ║
║  ██║     ██║     ███████║██║     █████╔╝ ███████╗                     ║
║  ██║     ██║     ██╔══██║██║     ██╔═██╗ ╚════██║                     ║
║  ╚██████╗███████╗██║  ██║╚██████╗██║  ██╗███████║                     ║
║   ╚═════╝╚══════╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝╚══════╝                     ║
║                                                                       ║
║         V U L N E R A B I L I T Y   T A X O N O M Y                   ║
║              C R O S S W A L K   S Y S T E M                          ║
║                                                                       ║
╚═══════════════════════════════════════════════════════════════════════╝
"#;

    for line in banner.lines() {
        println!("{}", line.color(CRT_GREEN));
    }
    add_scan_lines();
}

/// Print section header
pub fn print_section_header(title: &str) {
    println!();
    println!("{}", "═".repeat(75).color(CRT_GREEN));
    println!("{}", format!(" ▶ {}", title).color(CRT_AMBER).bold());
    println!("{}", "═".repeat(75).color(CRT_GREEN));
    println!();
}

/// Print a separator
pub fn print_separator() {
    println!("{}", "─".repeat(75).color(CRT_DIM));
}

/// Add CRT scan line effect
fn add_scan_lines() {
    println!("{}", "░".repeat(75).color(CRT_DIM));
}

/// Print info message
pub fn print_info(msg: &str) {
    println!("{} {}", "[INFO]".color(CRT_BLUE).bold(), msg.color(CRT_GREEN));
}

/// Print success message
pub fn print_success(msg: &str) {
    println!("{} {}", "[✓]".color(CRT_GREEN).bold(), msg.color(CRT_GREEN));
}

/// Print error message
pub fn print_error(msg: &str) {
    eprintln!("{} {}", "[ERROR]".color(CRT_RED).bold(), msg.color(CRT_RED));
}

/// Print warning message
pub fn print_warning(msg: &str) {
    println!("{} {}", "[!]".color(CRT_AMBER).bold(), msg.color(CRT_AMBER));
}

/// Print compact view of a concept
pub fn print_compact_view(concept: &Concept, index: usize) -> Result<()> {
    println!("{}", format!("┌─ Result #{} ", index).color(CRT_GREEN).bold());
    println!("│");
    println!("│ {}: {}", "Title".color(CRT_AMBER).bold(), concept.title.color(CRT_GREEN));
    println!("│ {}: {}", "Slug".color(CRT_AMBER).bold(), concept.slug.color(CRT_BLUE));
    
    if concept.is_specifier {
        println!("│ {}: {}", "Type".color(CRT_AMBER).bold(), "Specifier".color(CRT_AMBER));
    }
    
    if !concept.aliases.is_empty() {
        let aliases = concept.aliases.join(", ");
        println!("│ {}: {}", "Aliases".color(CRT_AMBER).bold(), aliases.color(CRT_DIM));
    }
    
    // Show hierarchical slugs
    let hierarchy = concept.generate_hierarchical_slugs();
    if hierarchy.len() > 1 {
        println!("│ {}: {}", "Path".color(CRT_AMBER).bold(), hierarchy.join(" → ").color(CRT_BLUE));
    }
    
    println!("└{}", "─".repeat(73).color(CRT_GREEN));
    
    Ok(())
}

/// Print detailed view of a concept
pub fn print_detailed_view(concept: &Concept, index: usize) -> Result<()> {
    println!("{}", format!("╔═ Result #{} ", index).color(CRT_GREEN).bold());
    println!("║");
    println!("║ {}: {}", "Title".color(CRT_AMBER).bold(), concept.title.color(CRT_GREEN).bold());
    println!("║ {}: {}", "ID".color(CRT_AMBER).bold(), concept.id.color(CRT_BLUE));
    println!("║ {}: {}", "Slug".color(CRT_AMBER).bold(), concept.slug.color(CRT_BLUE));
    println!("║ {}: {}", "Type".color(CRT_AMBER).bold(), 
             if concept.is_specifier { "Specifier".color(CRT_AMBER) } else { "Vulnerability".color(CRT_GREEN) });
    
    println!("║");
    
    // Show hierarchical path
    let hierarchy = concept.generate_hierarchical_slugs();
    println!("║ {}:", "Hierarchical Path".color(CRT_AMBER).bold());
    for (idx, slug) in hierarchy.iter().enumerate() {
        let indent = "  ".repeat(idx);
        let connector = if idx == hierarchy.len() - 1 { "└─" } else { "├─" };
        println!("║   {}{} {}", indent, connector.color(CRT_DIM), slug.color(CRT_GREEN));
    }
    println!("║");
    
    if !concept.aliases.is_empty() {
        println!("║ {}:", "Aliases".color(CRT_AMBER).bold());
        for alias in &concept.aliases {
            println!("║   • {}", alias.color(CRT_GREEN));
        }
        println!("║");
    }
    
    if !concept.mappings.is_empty() {
        println!("║ {}:", "Cross-Taxonomy Mappings".color(CRT_AMBER).bold());
        for (taxonomy, mappings) in &concept.mappings {
            println!("║   {} →", taxonomy.to_uppercase().color(CRT_AMBER).bold());
            for mapping in mappings {
                println!("║     ├─ {}", mapping.id.color(CRT_GREEN));
                println!("║     │  Confidence: {}", mapping.confidence.color(CRT_BLUE));
                if let Some(note) = &mapping.note {
                    for line in textwrap::wrap(note, 64) {
                        println!("║     │  Note: {}", line.color(CRT_DIM));
                    }
                }
            }
        }
        println!("║");
    }
    
    // Show all hierarchical slugs from all taxonomies
    let all_slugs = concept.get_all_taxonomy_slugs();
    if !all_slugs.is_empty() {
        println!("║ {}:", "All Taxonomy Slugs (for tagging)".color(CRT_AMBER).bold());
        for slug in all_slugs {
            println!("║   [{}]", slug.color(CRT_BLUE));
        }
    }
    
    println!("╚{}", "═".repeat(73).color(CRT_GREEN));
    
    Ok(())
}

/// Print tree view of a concept hierarchy
pub fn print_tree_view(concept: &Concept, manager: &ConceptManager) -> Result<()> {
    println!();
    println!("{}", "┌─ TAXONOMY TREE".color(CRT_GREEN).bold());
    println!("│");
    
    // Build hierarchy from root
    let hierarchy = concept.get_hierarchy();
    let mut current_path = String::new();
    
    for (depth, part) in hierarchy.iter().enumerate() {
        if !current_path.is_empty() {
            current_path.push('/');
        }
        current_path.push_str(part);
        
        let indent = "│   ".repeat(depth);
        let connector = if depth == hierarchy.len() - 1 {
            "└──"
        } else {
            "├──"
        };
        
        let node_concept = manager.get_by_slug(&current_path);
        let title = node_concept
            .as_ref()
            .map(|c| c.title.as_str())
            .unwrap_or(part);
        
        let color = if depth == hierarchy.len() - 1 {
            CRT_AMBER
        } else {
            CRT_GREEN
        };
        
        println!("│{}{} {} {}", 
                 indent.color(CRT_DIM),
                 connector.color(color),
                 title.color(color).bold(),
                 format!("({})", part).color(CRT_DIM));
        
        // Show children of the target concept
        if depth == hierarchy.len() - 1 {
            let children = manager.get_children(&current_path);
            if !children.is_empty() {
                println!("│{}│", "│   ".repeat(depth).color(CRT_DIM));
                for (idx, child) in children.iter().enumerate() {
                    let child_indent = "│   ".repeat(depth + 1);
                    let child_connector = if idx == children.len() - 1 { "└──" } else { "├──" };
                    let child_type = if child.is_specifier { " [SPEC]" } else { "" };
                    println!("│{}{} {}{}", 
                             child_indent.color(CRT_DIM),
                             child_connector.color(CRT_DIM),
                             child.title.color(CRT_GREEN),
                             child_type.color(CRT_AMBER));
                }
            }
        }
    }
    
    println!("│");
    println!("{}{}", "└".color(CRT_GREEN), "─".repeat(73).color(CRT_GREEN));
    
    Ok(())
}

/// Print full details of a concept
pub fn print_full_details(concept: &Concept) -> Result<()> {
    print_detailed_view(concept, 1)
}

/// Print list view of concepts
pub fn print_list_view(concepts: &[Concept]) -> Result<()> {
    println!("{} concepts found", concepts.len());
    println!();
    
    for (idx, concept) in concepts.iter().enumerate() {
        let spec_marker = if concept.is_specifier { " [SPEC]" } else { "" };
        println!("{:3}. {}{} {}", 
                 format!("{}", idx + 1).color(CRT_DIM),
                 concept.title.color(CRT_GREEN),
                 spec_marker.color(CRT_AMBER),
                 format!("({})", concept.slug).color(CRT_BLUE));
    }
    
    Ok(())
}

/// Print tree listing of all concepts
pub fn print_tree_listing(concepts: &[Concept], manager: &ConceptManager) -> Result<()> {
    println!("{}", "ROOT".color(CRT_AMBER).bold());
    
    // Get root concepts
    let roots = manager.get_root_concepts();
    
    for root in roots {
        print_tree_recursive(root, manager, 0)?;
    }
    
    Ok(())
}

fn print_tree_recursive(concept: &Concept, manager: &ConceptManager, depth: usize) -> Result<()> {
    let indent = "│   ".repeat(depth);
    let connector = "├──";
    let spec_marker = if concept.is_specifier { " [SPEC]" } else { "" };
    
    println!("{}{} {}{}", 
             indent.color(CRT_DIM),
             connector.color(CRT_GREEN),
             concept.title.color(CRT_GREEN),
             spec_marker.color(CRT_AMBER));
    
    let children = manager.get_children(&concept.slug);
    for child in children {
        print_tree_recursive(child, manager, depth + 1)?;
    }
    
    Ok(())
}

/// Print footer
pub fn print_footer() {
    println!();
    add_scan_lines();
    println!("{}", 
             "[ clacks v0.1.0 | VASS Triager Tool | Status: OPERATIONAL ]"
             .color(CRT_GREEN)
             .dimmed());
    println!();
}

// Helper to wrap text - simple implementation
mod textwrap {
    pub fn wrap(text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();
        
        for word in words {
            if current_line.len() + word.len() + 1 > width {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        if lines.is_empty() {
            lines.push(text.to_string());
        }
        
        lines
    }
}

