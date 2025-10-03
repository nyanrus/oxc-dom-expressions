//! Static analysis and optimization utilities for DOM expressions

use crate::template::Template;
use std::collections::HashMap;

/// Statistics about templates generated during transformation
#[derive(Debug, Clone, Default)]
pub struct TemplateStats {
    /// Total number of templates encountered
    pub total_templates: usize,
    /// Number of unique templates (after deduplication)
    pub unique_templates: usize,
    /// Number of templates reused
    pub reused_templates: usize,
    /// Total HTML size before deduplication
    pub total_html_size: usize,
    /// Total HTML size after deduplication
    pub deduplicated_html_size: usize,
    /// Number of static templates (no dynamic slots)
    pub static_templates: usize,
    /// Number of dynamic templates
    pub dynamic_templates: usize,
}

impl TemplateStats {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate space saved by deduplication
    pub fn space_saved(&self) -> usize {
        self.total_html_size.saturating_sub(self.deduplicated_html_size)
    }

    /// Calculate deduplication ratio (0.0 to 1.0)
    pub fn deduplication_ratio(&self) -> f64 {
        if self.total_templates == 0 {
            return 0.0;
        }
        (self.total_templates - self.unique_templates) as f64 / self.total_templates as f64
    }

    /// Calculate average template size
    pub fn average_template_size(&self) -> f64 {
        if self.unique_templates == 0 {
            return 0.0;
        }
        self.deduplicated_html_size as f64 / self.unique_templates as f64
    }
}

/// Template optimizer for static analysis
pub struct TemplateOptimizer {
    /// Map of template HTML to usage count
    template_usage: HashMap<String, usize>,
    /// Map of template HTML to template data
    templates: HashMap<String, Template>,
}

impl TemplateOptimizer {
    /// Create a new template optimizer
    pub fn new() -> Self {
        Self {
            template_usage: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    /// Record a template usage
    pub fn record_template(&mut self, template: Template) {
        let html = template.html.clone();
        let count = self.template_usage.entry(html.clone()).or_insert(0);
        *count += 1;
        self.templates.entry(html).or_insert(template);
    }

    /// Get statistics about template usage
    pub fn get_stats(&self) -> TemplateStats {
        let mut stats = TemplateStats::new();
        
        stats.unique_templates = self.templates.len();
        stats.total_templates = self.template_usage.values().sum();
        stats.reused_templates = stats.total_templates.saturating_sub(stats.unique_templates);
        
        for (html, template) in &self.templates {
            let usage_count = self.template_usage.get(html).unwrap_or(&0);
            
            // Count total size (if this template was not deduplicated)
            stats.total_html_size += html.len() * usage_count;
            
            // Count deduplicated size (template only stored once)
            stats.deduplicated_html_size += html.len();
            
            // Count static vs dynamic
            if template.dynamic_slots.is_empty() {
                stats.static_templates += 1;
            } else {
                stats.dynamic_templates += 1;
            }
        }
        
        stats
    }

    /// Get templates that are used multiple times (good deduplication targets)
    pub fn get_reused_templates(&self) -> Vec<(String, usize)> {
        self.template_usage
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(html, &count)| (html.clone(), count))
            .collect()
    }

    /// Find optimization opportunities
    pub fn find_optimizations(&self) -> Vec<Optimization> {
        let mut optimizations = Vec::new();
        
        // Check for large templates that could be split
        for (html, template) in &self.templates {
            if html.len() > 1000 && template.dynamic_slots.len() > 5 {
                optimizations.push(Optimization {
                    kind: OptimizationKind::LargeTemplate,
                    message: format!(
                        "Large template ({} bytes, {} dynamic slots) could be split",
                        html.len(),
                        template.dynamic_slots.len()
                    ),
                    template_html: html.clone(),
                });
            }
        }
        
        // Check for templates with many dynamic slots
        for (html, template) in &self.templates {
            if template.dynamic_slots.len() > 10 {
                optimizations.push(Optimization {
                    kind: OptimizationKind::ManyDynamicSlots,
                    message: format!(
                        "Template with {} dynamic slots may have performance impact",
                        template.dynamic_slots.len()
                    ),
                    template_html: html.clone(),
                });
            }
        }
        
        optimizations
    }
}

impl Default for TemplateOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Optimization suggestion
#[derive(Debug, Clone)]
pub struct Optimization {
    /// Type of optimization
    pub kind: OptimizationKind,
    /// Description of the optimization
    pub message: String,
    /// Template HTML this applies to
    pub template_html: String,
}

/// Type of optimization that can be applied
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationKind {
    /// Template is very large and could be split
    LargeTemplate,
    /// Template has many dynamic slots
    ManyDynamicSlots,
    /// Template is used frequently (good deduplication candidate)
    FrequentlyUsed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{DynamicSlot, SlotType};

    #[test]
    fn test_template_stats_empty() {
        let stats = TemplateStats::new();
        assert_eq!(stats.total_templates, 0);
        assert_eq!(stats.unique_templates, 0);
        assert_eq!(stats.deduplication_ratio(), 0.0);
    }

    #[test]
    fn test_template_stats_calculations() {
        let stats = TemplateStats {
            total_templates: 10,
            unique_templates: 5,
            reused_templates: 5,
            total_html_size: 1000,
            deduplicated_html_size: 500,
            static_templates: 2,
            dynamic_templates: 3,
        };
        
        assert_eq!(stats.space_saved(), 500);
        assert_eq!(stats.deduplication_ratio(), 0.5);
        assert_eq!(stats.average_template_size(), 100.0);
    }

    #[test]
    fn test_optimizer_record_template() {
        let mut optimizer = TemplateOptimizer::new();
        
        let template1 = Template {
            html: "<div>Hello</div>".to_string(),
            dynamic_slots: vec![],
        };
        
        let template2 = Template {
            html: "<div>Hello</div>".to_string(),
            dynamic_slots: vec![],
        };
        
        optimizer.record_template(template1);
        optimizer.record_template(template2);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.total_templates, 2);
        assert_eq!(stats.unique_templates, 1);
        assert_eq!(stats.reused_templates, 1);
    }

    #[test]
    fn test_optimizer_static_vs_dynamic() {
        let mut optimizer = TemplateOptimizer::new();
        
        let static_template = Template {
            html: "<div>Static</div>".to_string(),
            dynamic_slots: vec![],
        };
        
        let dynamic_template = Template {
            html: "<div>Dynamic</div>".to_string(),
            dynamic_slots: vec![DynamicSlot {
                path: vec![],
                slot_type: SlotType::TextContent,
                marker_path: None,
            }],
        };
        
        optimizer.record_template(static_template);
        optimizer.record_template(dynamic_template);
        
        let stats = optimizer.get_stats();
        assert_eq!(stats.static_templates, 1);
        assert_eq!(stats.dynamic_templates, 1);
    }

    #[test]
    fn test_optimizer_find_large_templates() {
        let mut optimizer = TemplateOptimizer::new();
        
        let large_html = "x".repeat(1500);
        let large_template = Template {
            html: large_html,
            dynamic_slots: (0..6)
                .map(|_| DynamicSlot {
                    path: vec![],
                    slot_type: SlotType::TextContent,
                    marker_path: None,
                })
                .collect(),
        };
        
        optimizer.record_template(large_template);
        
        let optimizations = optimizer.find_optimizations();
        assert_eq!(optimizations.len(), 1);
        assert_eq!(optimizations[0].kind, OptimizationKind::LargeTemplate);
    }

    #[test]
    fn test_optimizer_find_many_slots() {
        let mut optimizer = TemplateOptimizer::new();
        
        let template = Template {
            html: "<div>Many slots</div>".to_string(),
            dynamic_slots: (0..15)
                .map(|_| DynamicSlot {
                    path: vec![],
                    slot_type: SlotType::TextContent,
                    marker_path: None,
                })
                .collect(),
        };
        
        optimizer.record_template(template);
        
        let optimizations = optimizer.find_optimizations();
        assert_eq!(optimizations.len(), 1);
        assert_eq!(optimizations[0].kind, OptimizationKind::ManyDynamicSlots);
    }
}
