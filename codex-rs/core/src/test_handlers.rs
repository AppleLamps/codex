//! Test tool handlers for processing AI agent test requests.
//!
//! This module implements the actual logic for handling test tool calls,
//! managing test execution, and formatting responses for the AI agent.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use tracing::{debug, error, info, warn};

use crate::models::FunctionCallOutputPayload;
use crate::test_runner::{TestRunner, TestResult, TestStatus};
use crate::test_tools::*;

/// Test manager that handles test execution and analysis
pub struct TestManager {
    test_runner: TestRunner,
    workspace_root: PathBuf,
}

impl TestManager {
    /// Create a new test manager
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            test_runner: TestRunner::new(workspace_root.clone()),
            workspace_root,
        }
    }

    /// Get the workspace root
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }
}

/// Handle test.run requests
pub async fn handle_test_run(
    args: TestRunArgs,
    test_manager: &TestManager,
) -> Result<FunctionCallOutputPayload> {
    info!("Running tests with args: {:?}", args);

    // Set working directory
    let working_dir = if let Some(dir) = &args.working_dir {
        PathBuf::from(dir)
    } else {
        test_manager.workspace_root().to_path_buf()
    };

    // Create a test runner for the specified directory
    let test_runner = TestRunner::new(working_dir);

    // Run the tests
    match test_runner.run_tests(
        args.framework.as_deref(),
        args.filter.as_deref(),
        args.coverage.unwrap_or(false),
    ).await {
        Ok(test_result) => {
            let response = convert_test_result_to_response(test_result);
            
            Ok(FunctionCallOutputPayload {
                content: serde_json::to_string_pretty(&response)?,
                success: Some(response.success),
            })
        }
        Err(e) => {
            error!("Test execution failed: {}", e);
            Ok(FunctionCallOutputPayload {
                content: format!("Test execution failed: {}", e),
                success: Some(false),
            })
        }
    }
}

/// Handle test.analyze requests
pub async fn handle_test_analyze(
    args: TestAnalyzeArgs,
    test_manager: &TestManager,
) -> Result<FunctionCallOutputPayload> {
    info!("Analyzing tests with args: {:?}", args);

    let path = Path::new(&args.path);
    
    match args.analysis_type.as_str() {
        "structure" => {
            let analysis = analyze_test_structure(path, args.framework.as_deref()).await?;
            Ok(FunctionCallOutputPayload {
                content: serde_json::to_string_pretty(&analysis)?,
                success: Some(true),
            })
        }
        "coverage" => {
            let analysis = analyze_test_coverage(path, test_manager).await?;
            Ok(FunctionCallOutputPayload {
                content: serde_json::to_string_pretty(&analysis)?,
                success: Some(true),
            })
        }
        "performance" => {
            let analysis = analyze_test_performance(path, args.framework.as_deref()).await?;
            Ok(FunctionCallOutputPayload {
                content: serde_json::to_string_pretty(&analysis)?,
                success: Some(true),
            })
        }
        _ => {
            Ok(FunctionCallOutputPayload {
                content: format!("Unknown analysis type: {}. Supported types: structure, coverage, performance", args.analysis_type),
                success: Some(false),
            })
        }
    }
}

/// Handle test.generate requests
pub async fn handle_test_generate(
    args: TestGenerateArgs,
    test_manager: &TestManager,
) -> Result<FunctionCallOutputPayload> {
    info!("Generating tests with args: {:?}", args);

    let source_file = Path::new(&args.source_file);
    
    if !source_file.exists() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Source file not found: {}", args.source_file),
            success: Some(false),
        });
    }

    match generate_tests_for_file(source_file, &args).await {
        Ok(response) => {
            Ok(FunctionCallOutputPayload {
                content: serde_json::to_string_pretty(&response)?,
                success: Some(true),
            })
        }
        Err(e) => {
            error!("Test generation failed: {}", e);
            Ok(FunctionCallOutputPayload {
                content: format!("Test generation failed: {}", e),
                success: Some(false),
            })
        }
    }
}

/// Handle test.debug requests
pub async fn handle_test_debug(
    args: TestDebugArgs,
    test_manager: &TestManager,
) -> Result<FunctionCallOutputPayload> {
    info!("Debugging test with args: {:?}", args);

    match debug_test(&args, test_manager).await {
        Ok(response) => {
            Ok(FunctionCallOutputPayload {
                content: serde_json::to_string_pretty(&response)?,
                success: Some(true),
            })
        }
        Err(e) => {
            error!("Test debugging failed: {}", e);
            Ok(FunctionCallOutputPayload {
                content: format!("Test debugging failed: {}", e),
                success: Some(false),
            })
        }
    }
}

/// Convert TestResult to TestRunResponse
fn convert_test_result_to_response(test_result: TestResult) -> TestRunResponse {
    let pass_rate = if test_result.total_tests > 0 {
        (test_result.passed as f64 / test_result.total_tests as f64) * 100.0
    } else {
        0.0
    };

    let test_cases = test_result.test_cases.into_iter().map(|tc| {
        TestCaseResult {
            name: tc.name,
            status: match tc.status {
                TestStatus::Passed => "passed".to_string(),
                TestStatus::Failed => "failed".to_string(),
                TestStatus::Skipped => "skipped".to_string(),
                TestStatus::Ignored => "ignored".to_string(),
            },
            duration_ms: tc.duration.map(|d| d.as_millis() as u64),
            error: tc.error,
            file_path: tc.file_path,
            line_number: tc.line_number,
        }
    }).collect();

    let coverage = test_result.coverage.map(|cov| {
        CoverageReport {
            overall_percentage: cov.overall_percentage,
            line_coverage: cov.line_percentage,
            branch_coverage: cov.branch_percentage,
            function_coverage: cov.function_percentage,
            files: cov.file_coverage.into_iter().map(|fc| {
                FileCoverageReport {
                    file_path: fc.file_path,
                    percentage: fc.percentage,
                    lines_covered: fc.lines_covered,
                    total_lines: fc.total_lines,
                    uncovered_ranges: fc.uncovered_lines.windows(2).map(|w| {
                        LineRange { start: w[0], end: w[1] }
                    }).collect(),
                }
            }).collect(),
        }
    });

    TestRunResponse {
        success: test_result.success,
        framework: "auto-detected".to_string(), // TODO: Get actual framework
        summary: TestSummary {
            total: test_result.total_tests,
            passed: test_result.passed,
            failed: test_result.failed,
            skipped: test_result.skipped,
            pass_rate,
        },
        test_cases,
        coverage,
        duration_ms: test_result.duration.as_millis() as u64,
        messages: test_result.errors,
    }
}

/// Analyze test structure
async fn analyze_test_structure(
    path: &Path,
    framework: Option<&str>,
) -> Result<TestAnalyzeResponse> {
    let mut metrics = HashMap::new();
    let mut recommendations = Vec::new();

    // Basic structure analysis
    if path.is_file() {
        // Analyze single test file
        let content = tokio::fs::read_to_string(path).await?;
        let line_count = content.lines().count();
        metrics.insert("line_count".to_string(), serde_json::Value::Number(line_count.into()));
        
        // Count test functions (simplified)
        let test_count = content.lines().filter(|line| {
            line.contains("test") || line.contains("it(") || line.contains("describe(")
        }).count();
        metrics.insert("test_count".to_string(), serde_json::Value::Number(test_count.into()));

        if test_count == 0 {
            recommendations.push("No tests found in file. Consider adding test functions.".to_string());
        }
    } else if path.is_dir() {
        // Analyze test directory
        let mut test_files = 0;
        let mut total_tests = 0;

        if let Ok(entries) = tokio::fs::read_dir(path).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let file_path = entry.path();
                if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                    if name.contains("test") || name.ends_with("_test.rs") || name.ends_with(".test.js") {
                        test_files += 1;
                        
                        if let Ok(content) = tokio::fs::read_to_string(&file_path).await {
                            let test_count = content.lines().filter(|line| {
                                line.contains("test") || line.contains("it(") || line.contains("describe(")
                            }).count();
                            total_tests += test_count;
                        }
                    }
                }
            }
        }

        metrics.insert("test_files".to_string(), serde_json::Value::Number(test_files.into()));
        metrics.insert("total_tests".to_string(), serde_json::Value::Number(total_tests.into()));

        if test_files == 0 {
            recommendations.push("No test files found. Consider adding test files.".to_string());
        }
    }

    Ok(TestAnalyzeResponse {
        analysis_type: "structure".to_string(),
        results: serde_json::Value::Object(serde_json::Map::new()),
        recommendations,
        metrics,
    })
}

/// Analyze test coverage
async fn analyze_test_coverage(
    path: &Path,
    test_manager: &TestManager,
) -> Result<TestAnalyzeResponse> {
    // This would typically run tests with coverage and analyze the results
    // For now, return a placeholder response
    let mut metrics = HashMap::new();
    let mut recommendations = Vec::new();

    metrics.insert("coverage_available".to_string(), serde_json::Value::Bool(false));
    recommendations.push("Run tests with coverage enabled to get detailed coverage analysis.".to_string());

    Ok(TestAnalyzeResponse {
        analysis_type: "coverage".to_string(),
        results: serde_json::Value::Object(serde_json::Map::new()),
        recommendations,
        metrics,
    })
}

/// Analyze test performance
async fn analyze_test_performance(
    path: &Path,
    framework: Option<&str>,
) -> Result<TestAnalyzeResponse> {
    // This would analyze test execution times and performance metrics
    // For now, return a placeholder response
    let mut metrics = HashMap::new();
    let mut recommendations = Vec::new();

    metrics.insert("performance_analysis_available".to_string(), serde_json::Value::Bool(false));
    recommendations.push("Run tests with performance profiling to get detailed performance analysis.".to_string());

    Ok(TestAnalyzeResponse {
        analysis_type: "performance".to_string(),
        results: serde_json::Value::Object(serde_json::Map::new()),
        recommendations,
        metrics,
    })
}

/// Generate tests for a source file
async fn generate_tests_for_file(
    source_file: &Path,
    args: &TestGenerateArgs,
) -> Result<TestGenerateResponse> {
    let file_name = source_file.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let extension = source_file.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // Generate basic test template based on language
    let (test_content, test_file_name) = match extension {
        "rs" => {
            let content = format!(r#"#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{}_basic() {{
        // TODO: Add test implementation
        assert!(true);
    }}

    #[test]
    fn test_{}_edge_cases() {{
        // TODO: Add edge case tests
        assert!(true);
    }}
}}
"#, file_name, file_name);
            (content, format!("{}_test.rs", file_name))
        }
        "js" | "ts" => {
            let content = format!(r#"import {{ describe, it, expect }} from 'jest';
// import {{ {} }} from './{}';

describe('{}', () => {{
    it('should work correctly', () => {{
        // TODO: Add test implementation
        expect(true).toBe(true);
    }});

    it('should handle edge cases', () => {{
        // TODO: Add edge case tests
        expect(true).toBe(true);
    }});
}});
"#, file_name, file_name, file_name);
            (content, format!("{}.test.{}", file_name, extension))
        }
        "py" => {
            let content = format!(r#"import unittest
# from {} import *

class Test{}(unittest.TestCase):
    def test_basic_functionality(self):
        """Test basic functionality."""
        # TODO: Add test implementation
        self.assertTrue(True)

    def test_edge_cases(self):
        """Test edge cases."""
        # TODO: Add edge case tests
        self.assertTrue(True)

if __name__ == '__main__':
    unittest.main()
"#, file_name, file_name.to_uppercase());
            (content, format!("test_{}.py", file_name))
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported file type: {}", extension));
        }
    };

    let output_dir = if let Some(dir) = &args.output_dir {
        PathBuf::from(dir)
    } else {
        source_file.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    let test_file_path = output_dir.join(&test_file_name);

    Ok(TestGenerateResponse {
        generated_files: vec![GeneratedTestFile {
            file_path: test_file_path.to_string_lossy().to_string(),
            content: test_content,
            description: format!("Generated {} tests for {}", args.test_type, source_file.display()),
        }],
        framework: args.framework.clone().unwrap_or_else(|| "auto".to_string()),
        test_type: args.test_type.clone(),
        instructions: format!("Generated test file: {}. Please review and implement the test cases.", test_file_name),
    })
}

/// Debug a specific test
async fn debug_test(
    args: &TestDebugArgs,
    test_manager: &TestManager,
) -> Result<TestDebugResponse> {
    // This would run the test in debug mode and collect detailed information
    // For now, return a placeholder response
    
    Ok(TestDebugResponse {
        test_name: args.test_name.clone(),
        debug_info: DebugInfo {
            trace: vec!["Debug trace not yet implemented".to_string()],
            variables: HashMap::new(),
            stack_trace: None,
            performance: None,
        },
        suggestions: vec![
            "Enable verbose test output for more details".to_string(),
            "Check test assertions and expected values".to_string(),
            "Verify test setup and teardown procedures".to_string(),
        ],
    })
}
