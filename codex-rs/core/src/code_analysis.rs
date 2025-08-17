//! Code analysis tools for static analysis and quality assessment.
//!
//! This module provides comprehensive code analysis capabilities including
//! complexity metrics, dependency analysis, and code quality assessment.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Code analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisConfig {
    /// Languages to analyze
    pub languages: Vec<String>,
    /// Analysis types to perform
    pub analysis_types: Vec<AnalysisType>,
    /// Maximum file size to analyze (in bytes)
    pub max_file_size: usize,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
}

/// Types of code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisType {
    Complexity,
    Dependencies,
    Quality,
    Security,
    Performance,
    Documentation,
}

/// Code analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    /// File path analyzed
    pub file_path: String,
    /// Language detected
    pub language: String,
    /// Analysis metrics
    pub metrics: CodeMetrics,
    /// Issues found
    pub issues: Vec<CodeIssue>,
    /// Dependencies found
    pub dependencies: Vec<Dependency>,
    /// Overall quality score (0-100)
    pub quality_score: f64,
}

/// Code metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    /// Lines of code
    pub lines_of_code: u32,
    /// Lines of comments
    pub lines_of_comments: u32,
    /// Blank lines
    pub blank_lines: u32,
    /// Cyclomatic complexity
    pub cyclomatic_complexity: u32,
    /// Number of functions
    pub function_count: u32,
    /// Number of classes
    pub class_count: u32,
    /// Maximum function length
    pub max_function_length: u32,
    /// Average function length
    pub avg_function_length: f64,
}

/// Code issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    /// Issue type
    pub issue_type: IssueType,
    /// Severity level
    pub severity: IssueSeverity,
    /// Issue message
    pub message: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: Option<u32>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Types of code issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueType {
    Complexity,
    Style,
    Security,
    Performance,
    Maintainability,
    Documentation,
    Duplication,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,
    /// Version if available
    pub version: Option<String>,
    /// Dependency type (direct, dev, peer, etc.)
    pub dependency_type: String,
    /// Source (package.json, Cargo.toml, etc.)
    pub source: String,
    /// Whether it's outdated
    pub outdated: Option<bool>,
    /// Security vulnerabilities
    pub vulnerabilities: Vec<SecurityVulnerability>,
}

/// Security vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    /// Vulnerability ID
    pub id: String,
    /// Severity
    pub severity: String,
    /// Description
    pub description: String,
    /// Fixed version
    pub fixed_version: Option<String>,
}

/// Code analyzer
pub struct CodeAnalyzer {
    config: CodeAnalysisConfig,
}

impl CodeAnalyzer {
    /// Create a new code analyzer
    pub fn new(config: CodeAnalysisConfig) -> Self {
        Self { config }
    }

    /// Analyze a single file
    pub async fn analyze_file(&self, file_path: &Path) -> Result<CodeAnalysisResult> {
        let content = fs::read_to_string(file_path).await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        if content.len() > self.config.max_file_size {
            return Err(anyhow::anyhow!("File too large: {} bytes", content.len()));
        }

        let language = detect_language(file_path);
        let metrics = analyze_metrics(&content, &language);
        let issues = analyze_issues(&content, &language, file_path);
        let dependencies = analyze_dependencies(&content, &language, file_path).await;

        let quality_score = calculate_quality_score(&metrics, &issues);

        Ok(CodeAnalysisResult {
            file_path: file_path.to_string_lossy().to_string(),
            language,
            metrics,
            issues,
            dependencies,
            quality_score,
        })
    }

    /// Analyze a directory
    pub async fn analyze_directory(&self, dir_path: &Path) -> Result<Vec<CodeAnalysisResult>> {
        let mut results = Vec::new();
        let mut entries = fs::read_dir(dir_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    if self.should_analyze_file(&path, extension) {
                        match self.analyze_file(&path).await {
                            Ok(result) => results.push(result),
                            Err(e) => warn!("Failed to analyze {}: {}", path.display(), e),
                        }
                    }
                }
            } else if path.is_dir() && !self.should_exclude_directory(&path) {
                // Recursively analyze subdirectories
                match Box::pin(self.analyze_directory(&path)).await {
                    Ok(mut sub_results) => results.append(&mut sub_results),
                    Err(e) => warn!("Failed to analyze directory {}: {}", path.display(), e),
                }
            }
        }

        Ok(results)
    }

    /// Check if a file should be analyzed
    fn should_analyze_file(&self, file_path: &Path, extension: &str) -> bool {
        // Check if extension matches supported languages
        let supported_extensions = get_supported_extensions();
        if !supported_extensions.contains(&extension.to_lowercase().as_str()) {
            return false;
        }

        // Check exclude patterns
        let file_name = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        for pattern in &self.config.exclude_patterns {
            if wildmatch::WildMatch::new(pattern).matches(file_name) {
                return false;
            }
        }

        true
    }

    /// Check if a directory should be excluded
    fn should_exclude_directory(&self, dir_path: &Path) -> bool {
        let dir_name = dir_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        // Common directories to exclude
        let exclude_dirs = ["node_modules", "target", ".git", ".svn", "__pycache__", "build", "dist"];
        exclude_dirs.contains(&dir_name)
    }
}

/// Detect the programming language of a file
pub fn detect_language(file_path: &Path) -> String {
    if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
        match extension.to_lowercase().as_str() {
            "rs" => "rust".to_string(),
            "js" | "jsx" => "javascript".to_string(),
            "ts" | "tsx" => "typescript".to_string(),
            "py" | "pyi" => "python".to_string(),
            "go" => "go".to_string(),
            "java" => "java".to_string(),
            "c" => "c".to_string(),
            "cpp" | "cc" | "cxx" => "cpp".to_string(),
            "h" | "hpp" => "header".to_string(),
            "cs" => "csharp".to_string(),
            "rb" => "ruby".to_string(),
            "php" => "php".to_string(),
            "swift" => "swift".to_string(),
            "kt" => "kotlin".to_string(),
            "scala" => "scala".to_string(),
            _ => "unknown".to_string(),
        }
    } else {
        "unknown".to_string()
    }
}

/// Get supported file extensions
pub fn get_supported_extensions() -> Vec<&'static str> {
    vec![
        "rs", "js", "jsx", "ts", "tsx", "py", "pyi", "go", "java",
        "c", "cpp", "cc", "cxx", "h", "hpp", "cs", "rb", "php",
        "swift", "kt", "scala"
    ]
}

/// Analyze code metrics
pub fn analyze_metrics(content: &str, language: &str) -> CodeMetrics {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len() as u32;
    
    let mut lines_of_code = 0;
    let mut lines_of_comments = 0;
    let mut blank_lines = 0;
    let mut function_count = 0;
    let mut class_count = 0;

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            blank_lines += 1;
        } else if is_comment_line(trimmed, language) {
            lines_of_comments += 1;
        } else {
            lines_of_code += 1;
            
            // Simple function/class detection
            if is_function_declaration(trimmed, language) {
                function_count += 1;
            }
            if is_class_declaration(trimmed, language) {
                class_count += 1;
            }
        }
    }

    // Simple cyclomatic complexity calculation
    let cyclomatic_complexity = calculate_cyclomatic_complexity(content, language);

    CodeMetrics {
        lines_of_code,
        lines_of_comments,
        blank_lines,
        cyclomatic_complexity,
        function_count,
        class_count,
        max_function_length: 0, // TODO: Implement
        avg_function_length: if function_count > 0 { lines_of_code as f64 / function_count as f64 } else { 0.0 },
    }
}

/// Check if a line is a comment
fn is_comment_line(line: &str, language: &str) -> bool {
    match language {
        "rust" | "javascript" | "typescript" | "go" | "java" | "c" | "cpp" | "csharp" => {
            line.starts_with("//") || line.starts_with("/*") || line.starts_with("*")
        }
        "python" => line.starts_with("#"),
        "ruby" => line.starts_with("#"),
        _ => false,
    }
}

/// Check if a line is a function declaration
fn is_function_declaration(line: &str, language: &str) -> bool {
    match language {
        "rust" => line.contains("fn "),
        "javascript" | "typescript" => {
            line.contains("function ") || line.contains("=> ") || line.contains("() =>")
        }
        "python" => line.starts_with("def "),
        "go" => line.contains("func "),
        "java" | "csharp" => {
            line.contains("public ") || line.contains("private ") || line.contains("protected ")
        }
        _ => false,
    }
}

/// Check if a line is a class declaration
fn is_class_declaration(line: &str, language: &str) -> bool {
    match language {
        "rust" => line.contains("struct ") || line.contains("enum ") || line.contains("trait "),
        "javascript" | "typescript" => line.contains("class "),
        "python" => line.starts_with("class "),
        "java" | "csharp" => line.contains("class ") || line.contains("interface "),
        _ => false,
    }
}

/// Calculate cyclomatic complexity (simplified)
fn calculate_cyclomatic_complexity(content: &str, language: &str) -> u32 {
    let mut complexity = 1; // Base complexity
    
    let complexity_keywords = match language {
        "rust" => vec!["if", "else", "match", "while", "for", "loop"],
        "javascript" | "typescript" => vec!["if", "else", "switch", "while", "for", "catch"],
        "python" => vec!["if", "elif", "else", "while", "for", "except"],
        "go" => vec!["if", "else", "switch", "for", "select"],
        "java" | "csharp" => vec!["if", "else", "switch", "while", "for", "catch"],
        _ => vec!["if", "else", "while", "for"],
    };

    for keyword in complexity_keywords {
        complexity += content.matches(&format!(" {} ", keyword)).count() as u32;
        complexity += content.matches(&format!("\t{} ", keyword)).count() as u32;
        complexity += content.matches(&format!("{} ", keyword)).count() as u32;
    }

    complexity
}

/// Analyze code issues
pub fn analyze_issues(content: &str, language: &str, file_path: &Path) -> Vec<CodeIssue> {
    let mut issues = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let line_number = (line_num + 1) as u32;
        
        // Check for long lines
        if line.len() > 120 {
            issues.push(CodeIssue {
                issue_type: IssueType::Style,
                severity: IssueSeverity::Low,
                message: format!("Line too long: {} characters", line.len()),
                line: line_number,
                column: Some(120),
                suggestion: Some("Consider breaking this line into multiple lines".to_string()),
            });
        }

        // Check for TODO/FIXME comments
        if line.to_lowercase().contains("todo") || line.to_lowercase().contains("fixme") {
            issues.push(CodeIssue {
                issue_type: IssueType::Maintainability,
                severity: IssueSeverity::Info,
                message: "TODO/FIXME comment found".to_string(),
                line: line_number,
                column: None,
                suggestion: Some("Consider addressing this TODO item".to_string()),
            });
        }

        // Language-specific checks
        match language {
            "rust" => {
                if line.contains("unwrap()") {
                    issues.push(CodeIssue {
                        issue_type: IssueType::Security,
                        severity: IssueSeverity::Medium,
                        message: "Use of unwrap() can cause panics".to_string(),
                        line: line_number,
                        column: None,
                        suggestion: Some("Consider using expect() with a descriptive message or proper error handling".to_string()),
                    });
                }
            }
            "javascript" | "typescript" => {
                if line.contains("eval(") {
                    issues.push(CodeIssue {
                        issue_type: IssueType::Security,
                        severity: IssueSeverity::High,
                        message: "Use of eval() is dangerous".to_string(),
                        line: line_number,
                        column: None,
                        suggestion: Some("Avoid using eval() as it can execute arbitrary code".to_string()),
                    });
                }
            }
            _ => {}
        }
    }

    issues
}

/// Analyze dependencies
async fn analyze_dependencies(content: &str, language: &str, file_path: &Path) -> Vec<Dependency> {
    let mut dependencies = Vec::new();

    // Check for dependency files in the same directory
    if let Some(parent) = file_path.parent() {
        match language {
            "rust" => {
                let cargo_toml = parent.join("Cargo.toml");
                if cargo_toml.exists() {
                    if let Ok(cargo_content) = fs::read_to_string(cargo_toml).await {
                        dependencies.extend(parse_cargo_dependencies(&cargo_content));
                    }
                }
            }
            "javascript" | "typescript" => {
                let package_json = parent.join("package.json");
                if package_json.exists() {
                    if let Ok(package_content) = fs::read_to_string(package_json).await {
                        dependencies.extend(parse_package_json_dependencies(&package_content));
                    }
                }
            }
            "python" => {
                let requirements_txt = parent.join("requirements.txt");
                if requirements_txt.exists() {
                    if let Ok(req_content) = fs::read_to_string(requirements_txt).await {
                        dependencies.extend(parse_requirements_txt(&req_content));
                    }
                }
            }
            _ => {}
        }
    }

    dependencies
}

/// Parse Cargo.toml dependencies
fn parse_cargo_dependencies(content: &str) -> Vec<Dependency> {
    // Simplified parsing - in a real implementation, use a proper TOML parser
    Vec::new() // TODO: Implement proper parsing
}

/// Parse package.json dependencies
fn parse_package_json_dependencies(content: &str) -> Vec<Dependency> {
    // Simplified parsing - in a real implementation, use a proper JSON parser
    Vec::new() // TODO: Implement proper parsing
}

/// Parse requirements.txt
fn parse_requirements_txt(content: &str) -> Vec<Dependency> {
    // Simplified parsing
    Vec::new() // TODO: Implement proper parsing
}

/// Calculate overall quality score
fn calculate_quality_score(metrics: &CodeMetrics, issues: &Vec<CodeIssue>) -> f64 {
    let mut score: f64 = 100.0;

    // Deduct points for issues
    for issue in issues {
        match issue.severity {
            IssueSeverity::Critical => score -= 20.0,
            IssueSeverity::High => score -= 10.0,
            IssueSeverity::Medium => score -= 5.0,
            IssueSeverity::Low => score -= 2.0,
            IssueSeverity::Info => score -= 0.5,
        }
    }

    // Deduct points for high complexity
    if metrics.cyclomatic_complexity > 20 {
        score -= 15.0;
    } else if metrics.cyclomatic_complexity > 10 {
        score -= 5.0;
    }

    // Bonus for good comment ratio
    let total_lines = metrics.lines_of_code + metrics.lines_of_comments + metrics.blank_lines;
    if total_lines > 0 {
        let comment_ratio = metrics.lines_of_comments as f64 / total_lines as f64;
        if comment_ratio > 0.2 {
            score += 5.0;
        }
    }

    score.max(0.0).min(100.0)
}

/// Default code analysis configuration
pub fn default_analysis_config() -> CodeAnalysisConfig {
    CodeAnalysisConfig {
        languages: vec![
            "rust".to_string(),
            "javascript".to_string(),
            "typescript".to_string(),
            "python".to_string(),
            "go".to_string(),
            "java".to_string(),
        ],
        analysis_types: vec![
            AnalysisType::Complexity,
            AnalysisType::Quality,
            AnalysisType::Dependencies,
        ],
        max_file_size: 1024 * 1024, // 1MB
        exclude_patterns: vec![
            "*.min.js".to_string(),
            "*.test.*".to_string(),
            "*.spec.*".to_string(),
            "node_modules/*".to_string(),
            "target/*".to_string(),
        ],
    }
}
