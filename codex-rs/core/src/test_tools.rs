//! Test framework tools for AI agent testing capabilities.
//!
//! This module defines the tool schemas that expose test running functionality
//! to the AI agent, enabling comprehensive testing workflows.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::openai_tools::{JsonSchema, OpenAiTool, ResponsesApiTool};

/// Arguments for the test.run tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunArgs {
    /// Optional test framework to use (auto-detected if not specified)
    pub framework: Option<String>,
    /// Optional test filter/pattern to run specific tests
    pub filter: Option<String>,
    /// Whether to include code coverage analysis
    pub coverage: Option<bool>,
    /// Working directory for test execution (defaults to workspace root)
    pub working_dir: Option<String>,
    /// Additional environment variables
    pub env: Option<std::collections::HashMap<String, String>>,
}

/// Arguments for the test.analyze tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAnalyzeArgs {
    /// Path to test files or directory to analyze
    pub path: String,
    /// Type of analysis: "structure", "coverage", "performance"
    pub analysis_type: String,
    /// Optional framework hint for better analysis
    pub framework: Option<String>,
}

/// Arguments for the test.generate tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerateArgs {
    /// Path to the source file to generate tests for
    pub source_file: String,
    /// Type of tests to generate: "unit", "integration", "e2e"
    pub test_type: String,
    /// Test framework to use
    pub framework: Option<String>,
    /// Output directory for generated tests
    pub output_dir: Option<String>,
}

/// Arguments for the test.debug tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDebugArgs {
    /// Specific test name or pattern to debug
    pub test_name: String,
    /// Test framework
    pub framework: Option<String>,
    /// Debug mode: "verbose", "trace", "profile"
    pub debug_mode: Option<String>,
}

/// Creates the test.run tool definition
pub(crate) fn create_test_run_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "framework".to_string(),
        JsonSchema::String {
            description: Some("Test framework to use (cargo, jest, pytest, go, maven, gradle). Auto-detected if not specified.".to_string()),
        },
    );
    
    properties.insert(
        "filter".to_string(),
        JsonSchema::String {
            description: Some("Test filter or pattern to run specific tests (e.g., test name, file pattern)".to_string()),
        },
    );
    
    properties.insert(
        "coverage".to_string(),
        JsonSchema::Boolean {
            description: Some("Whether to include code coverage analysis in the test run".to_string()),
        },
    );
    
    properties.insert(
        "working_dir".to_string(),
        JsonSchema::String {
            description: Some("Working directory for test execution (defaults to workspace root)".to_string()),
        },
    );
    
    properties.insert(
        "env".to_string(),
        JsonSchema::Object {
            properties: BTreeMap::new(),
            required: None,
            additional_properties: Some(true),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "test.run".to_string(),
        description: "Run tests using the appropriate test framework. Automatically detects the test framework (cargo, jest, pytest, go, etc.) and executes tests with optional filtering and coverage analysis. Returns detailed test results including pass/fail status, execution time, and error details.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: None,
            additional_properties: Some(false),
        },
    })
}

/// Creates the test.analyze tool definition
pub(crate) fn create_test_analyze_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "path".to_string(),
        JsonSchema::String {
            description: Some("Path to test files or directory to analyze".to_string()),
        },
    );
    
    properties.insert(
        "analysis_type".to_string(),
        JsonSchema::String {
            description: Some("Type of analysis: 'structure' (test organization), 'coverage' (code coverage), 'performance' (test execution metrics)".to_string()),
        },
    );
    
    properties.insert(
        "framework".to_string(),
        JsonSchema::String {
            description: Some("Optional framework hint for better analysis (cargo, jest, pytest, etc.)".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "test.analyze".to_string(),
        description: "Analyze test files and test coverage. Provides insights into test structure, coverage gaps, test performance, and recommendations for improving test quality.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["path".to_string(), "analysis_type".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the test.generate tool definition
pub(crate) fn create_test_generate_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "source_file".to_string(),
        JsonSchema::String {
            description: Some("Path to the source file to generate tests for".to_string()),
        },
    );
    
    properties.insert(
        "test_type".to_string(),
        JsonSchema::String {
            description: Some("Type of tests to generate: 'unit' (function-level tests), 'integration' (module-level tests), 'e2e' (end-to-end tests)".to_string()),
        },
    );
    
    properties.insert(
        "framework".to_string(),
        JsonSchema::String {
            description: Some("Test framework to use for generated tests".to_string()),
        },
    );
    
    properties.insert(
        "output_dir".to_string(),
        JsonSchema::String {
            description: Some("Output directory for generated test files (defaults to standard test directory)".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "test.generate".to_string(),
        description: "Generate test templates and boilerplate for source files. Creates appropriate test structure based on the source code analysis and chosen test framework.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["source_file".to_string(), "test_type".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the test.debug tool definition
pub(crate) fn create_test_debug_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "test_name".to_string(),
        JsonSchema::String {
            description: Some("Specific test name or pattern to debug".to_string()),
        },
    );
    
    properties.insert(
        "framework".to_string(),
        JsonSchema::String {
            description: Some("Test framework being used".to_string()),
        },
    );
    
    properties.insert(
        "debug_mode".to_string(),
        JsonSchema::String {
            description: Some("Debug mode: 'verbose' (detailed output), 'trace' (execution trace), 'profile' (performance profiling)".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "test.debug".to_string(),
        description: "Debug failing tests with enhanced output and tracing. Provides detailed execution information to help identify the root cause of test failures.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["test_name".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Response structures for test tools

/// Response for test.run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunResponse {
    /// Whether the test run was successful overall
    pub success: bool,
    /// Test framework used
    pub framework: String,
    /// Test execution summary
    pub summary: TestSummary,
    /// Individual test results
    pub test_cases: Vec<TestCaseResult>,
    /// Coverage information if requested
    pub coverage: Option<CoverageReport>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Any errors or warnings
    pub messages: Vec<String>,
}

/// Test execution summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    /// Total number of tests
    pub total: u32,
    /// Number of passed tests
    pub passed: u32,
    /// Number of failed tests
    pub failed: u32,
    /// Number of skipped tests
    pub skipped: u32,
    /// Pass rate percentage
    pub pass_rate: f64,
}

/// Individual test case result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    /// Test name
    pub name: String,
    /// Test status
    pub status: String, // "passed", "failed", "skipped"
    /// Execution duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
    /// File path where test is located
    pub file_path: Option<String>,
    /// Line number of the test
    pub line_number: Option<u32>,
}

/// Coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Overall coverage percentage
    pub overall_percentage: f64,
    /// Line coverage
    pub line_coverage: f64,
    /// Branch coverage if available
    pub branch_coverage: Option<f64>,
    /// Function coverage if available
    pub function_coverage: Option<f64>,
    /// Per-file coverage details
    pub files: Vec<FileCoverageReport>,
}

/// Coverage report for individual files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverageReport {
    /// File path
    pub file_path: String,
    /// Coverage percentage
    pub percentage: f64,
    /// Lines covered
    pub lines_covered: u32,
    /// Total lines
    pub total_lines: u32,
    /// Uncovered line ranges
    pub uncovered_ranges: Vec<LineRange>,
}

/// Line range for coverage reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRange {
    /// Start line number
    pub start: u32,
    /// End line number
    pub end: u32,
}

/// Response for test.analyze
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAnalyzeResponse {
    /// Analysis type performed
    pub analysis_type: String,
    /// Analysis results
    pub results: serde_json::Value,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    /// Metrics and statistics
    pub metrics: std::collections::HashMap<String, serde_json::Value>,
}

/// Response for test.generate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerateResponse {
    /// Generated test files
    pub generated_files: Vec<GeneratedTestFile>,
    /// Test framework used
    pub framework: String,
    /// Test type generated
    pub test_type: String,
    /// Instructions for running the generated tests
    pub instructions: String,
}

/// Generated test file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTestFile {
    /// Path to the generated test file
    pub file_path: String,
    /// Test file content
    pub content: String,
    /// Description of what the test covers
    pub description: String,
}

/// Response for test.debug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestDebugResponse {
    /// Test name being debugged
    pub test_name: String,
    /// Debug information
    pub debug_info: DebugInfo,
    /// Suggested fixes
    pub suggestions: Vec<String>,
}

/// Debug information for failing tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugInfo {
    /// Execution trace
    pub trace: Vec<String>,
    /// Variable states at failure point
    pub variables: std::collections::HashMap<String, String>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// Performance metrics
    pub performance: Option<PerformanceMetrics>,
}

/// Performance metrics for test debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: Option<u64>,
    /// CPU usage percentage
    pub cpu_usage_percent: Option<f64>,
}
