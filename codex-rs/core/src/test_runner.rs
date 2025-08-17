//! Test framework integration for running and analyzing tests.
//!
//! This module provides comprehensive test running capabilities for various
//! programming languages and test frameworks, with intelligent result parsing
//! and reporting.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tracing::{debug, error, info, warn};

/// Test framework configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFrameworkConfig {
    /// Language this framework supports
    pub language: String,
    /// Framework name (e.g., "cargo", "jest", "pytest")
    pub framework: String,
    /// Command to run tests
    pub command: String,
    /// Arguments for the test command
    pub args: Vec<String>,
    /// File patterns that indicate this framework should be used
    pub detection_files: Vec<String>,
    /// Environment variables for the test command
    pub env: HashMap<String, String>,
}

/// Test execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Whether the test run was successful overall
    pub success: bool,
    /// Total number of tests run
    pub total_tests: u32,
    /// Number of passed tests
    pub passed: u32,
    /// Number of failed tests
    pub failed: u32,
    /// Number of skipped tests
    pub skipped: u32,
    /// Test execution duration
    pub duration: Duration,
    /// Individual test case results
    pub test_cases: Vec<TestCase>,
    /// Coverage information if available
    pub coverage: Option<CoverageInfo>,
    /// Raw output from the test runner
    pub raw_output: String,
    /// Parsed error messages
    pub errors: Vec<String>,
}

/// Individual test case result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test name or identifier
    pub name: String,
    /// Test status
    pub status: TestStatus,
    /// Test execution duration
    pub duration: Option<Duration>,
    /// Error message if failed
    pub error: Option<String>,
    /// File path where the test is located
    pub file_path: Option<String>,
    /// Line number of the test
    pub line_number: Option<u32>,
}

/// Test case status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Ignored,
}

/// Code coverage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageInfo {
    /// Overall coverage percentage
    pub overall_percentage: f64,
    /// Line coverage percentage
    pub line_percentage: f64,
    /// Branch coverage percentage if available
    pub branch_percentage: Option<f64>,
    /// Function coverage percentage if available
    pub function_percentage: Option<f64>,
    /// Per-file coverage details
    pub file_coverage: Vec<FileCoverage>,
}

/// Coverage information for a specific file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    /// File path
    pub file_path: String,
    /// Coverage percentage for this file
    pub percentage: f64,
    /// Lines covered
    pub lines_covered: u32,
    /// Total lines
    pub total_lines: u32,
    /// Uncovered line numbers
    pub uncovered_lines: Vec<u32>,
}

/// Test runner that can execute tests for various frameworks
pub struct TestRunner {
    frameworks: Vec<TestFrameworkConfig>,
    workspace_root: PathBuf,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            frameworks: default_test_frameworks(),
            workspace_root,
        }
    }

    /// Detect the appropriate test framework for the current workspace
    pub fn detect_framework(&self) -> Option<&TestFrameworkConfig> {
        for framework in &self.frameworks {
            for detection_file in &framework.detection_files {
                let file_path = self.workspace_root.join(detection_file);
                if file_path.exists() {
                    info!("Detected {} test framework via {}", framework.framework, detection_file);
                    return Some(framework);
                }
            }
        }
        None
    }

    /// Run tests using the detected or specified framework
    pub async fn run_tests(
        &self,
        framework: Option<&str>,
        test_filter: Option<&str>,
        include_coverage: bool,
    ) -> Result<TestResult> {
        let config = if let Some(framework_name) = framework {
            self.frameworks
                .iter()
                .find(|f| f.framework == framework_name)
                .ok_or_else(|| anyhow::anyhow!("Unknown test framework: {}", framework_name))?
        } else {
            self.detect_framework()
                .ok_or_else(|| anyhow::anyhow!("No test framework detected in workspace"))?
        };

        info!("Running tests with {} framework", config.framework);
        
        let start_time = Instant::now();
        
        // Build the command
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .current_dir(&self.workspace_root)
            .envs(&config.env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add test filter if specified
        if let Some(filter) = test_filter {
            match config.framework.as_str() {
                "cargo" => {
                    cmd.arg(filter);
                }
                "jest" | "npm" => {
                    cmd.args(&["--testNamePattern", filter]);
                }
                "pytest" => {
                    cmd.args(&["-k", filter]);
                }
                "go" => {
                    cmd.args(&["-run", filter]);
                }
                _ => {
                    warn!("Test filtering not implemented for framework: {}", config.framework);
                }
            }
        }

        // Add coverage flags if requested
        if include_coverage {
            match config.framework.as_str() {
                "cargo" => {
                    // For Rust, we might use tarpaulin or similar
                    cmd.env("CARGO_INCREMENTAL", "0");
                    cmd.env("RUSTFLAGS", "-Cinstrument-coverage");
                }
                "jest" => {
                    cmd.arg("--coverage");
                }
                "pytest" => {
                    cmd.args(&["--cov", "."]);
                }
                "go" => {
                    cmd.args(&["-cover", "-coverprofile=coverage.out"]);
                }
                _ => {
                    warn!("Coverage not implemented for framework: {}", config.framework);
                }
            }
        }

        // Execute the command
        let output = cmd.output().await
            .with_context(|| format!("Failed to execute test command: {}", config.command))?;

        let duration = start_time.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let raw_output = format!("{}\n{}", stdout, stderr);

        // Parse the test results based on the framework
        let test_result = self.parse_test_output(config, &raw_output, duration, output.status.success())
            .unwrap_or_else(|e| {
                warn!("Failed to parse test output: {}", e);
                TestResult {
                    success: output.status.success(),
                    total_tests: 0,
                    passed: 0,
                    failed: 0,
                    skipped: 0,
                    duration,
                    test_cases: vec![],
                    coverage: None,
                    raw_output: raw_output.to_string(),
                    errors: vec![format!("Failed to parse test output: {}", e)],
                }
            });

        Ok(test_result)
    }

    /// Parse test output based on the framework
    fn parse_test_output(
        &self,
        config: &TestFrameworkConfig,
        output: &str,
        duration: Duration,
        success: bool,
    ) -> Result<TestResult> {
        match config.framework.as_str() {
            "cargo" => self.parse_cargo_output(output, duration, success),
            "jest" => self.parse_jest_output(output, duration, success),
            "pytest" => self.parse_pytest_output(output, duration, success),
            "go" => self.parse_go_output(output, duration, success),
            _ => {
                // Generic parser for unknown frameworks
                Ok(TestResult {
                    success,
                    total_tests: 0,
                    passed: 0,
                    failed: 0,
                    skipped: 0,
                    duration,
                    test_cases: vec![],
                    coverage: None,
                    raw_output: output.to_string(),
                    errors: vec![],
                })
            }
        }
    }

    /// Parse Cargo test output
    fn parse_cargo_output(&self, output: &str, duration: Duration, success: bool) -> Result<TestResult> {
        let mut total_tests = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;
        let mut test_cases = Vec::new();
        let mut errors = Vec::new();

        for line in output.lines() {
            if line.contains("test result:") {
                // Parse summary line: "test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
                if let Some(summary) = line.split("test result:").nth(1) {
                    for part in summary.split(';') {
                        let part = part.trim();
                        if let Some(num_str) = part.split_whitespace().next() {
                            if let Ok(num) = num_str.parse::<u32>() {
                                if part.contains("passed") {
                                    passed = num;
                                } else if part.contains("failed") {
                                    failed = num;
                                } else if part.contains("ignored") {
                                    skipped = num;
                                }
                            }
                        }
                    }
                }
                total_tests = passed + failed + skipped;
            } else if line.starts_with("test ") && (line.contains("... ok") || line.contains("... FAILED")) {
                // Parse individual test results
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let test_name = parts[1].to_string();
                    let status = if line.contains("... ok") {
                        TestStatus::Passed
                    } else {
                        TestStatus::Failed
                    };

                    test_cases.push(TestCase {
                        name: test_name,
                        status,
                        duration: None,
                        error: None,
                        file_path: None,
                        line_number: None,
                    });
                }
            } else if line.contains("FAILED") && line.contains("::") {
                // Capture error information
                errors.push(line.to_string());
            }
        }

        Ok(TestResult {
            success,
            total_tests,
            passed,
            failed,
            skipped,
            duration,
            test_cases,
            coverage: None, // TODO: Parse coverage if available
            raw_output: output.to_string(),
            errors,
        })
    }

    /// Parse Jest test output
    fn parse_jest_output(&self, output: &str, duration: Duration, success: bool) -> Result<TestResult> {
        // Simplified Jest parser - would need more sophisticated parsing for real use
        let mut total_tests = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for line in output.lines() {
            if line.contains("Tests:") {
                // Parse Jest summary line
                if let Some(tests_part) = line.split("Tests:").nth(1) {
                    // Jest format: "Tests: 1 failed, 4 passed, 5 total"
                    for part in tests_part.split(',') {
                        let part = part.trim();
                        if let Some(num_str) = part.split_whitespace().next() {
                            if let Ok(num) = num_str.parse::<u32>() {
                                if part.contains("passed") {
                                    passed = num;
                                } else if part.contains("failed") {
                                    failed = num;
                                } else if part.contains("skipped") {
                                    skipped = num;
                                } else if part.contains("total") {
                                    total_tests = num;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(TestResult {
            success,
            total_tests,
            passed,
            failed,
            skipped,
            duration,
            test_cases: vec![], // TODO: Parse individual test cases
            coverage: None,     // TODO: Parse coverage if available
            raw_output: output.to_string(),
            errors: vec![],
        })
    }

    /// Parse pytest output
    fn parse_pytest_output(&self, output: &str, duration: Duration, success: bool) -> Result<TestResult> {
        // Simplified pytest parser
        let mut total_tests = 0;
        let mut passed = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for line in output.lines() {
            if line.contains("passed") || line.contains("failed") || line.contains("skipped") {
                // Parse pytest summary: "5 passed, 2 failed, 1 skipped in 2.34s"
                for part in line.split(',') {
                    let part = part.trim();
                    if let Some(num_str) = part.split_whitespace().next() {
                        if let Ok(num) = num_str.parse::<u32>() {
                            if part.contains("passed") {
                                passed = num;
                            } else if part.contains("failed") {
                                failed = num;
                            } else if part.contains("skipped") {
                                skipped = num;
                            }
                        }
                    }
                }
                total_tests = passed + failed + skipped;
            }
        }

        Ok(TestResult {
            success,
            total_tests,
            passed,
            failed,
            skipped,
            duration,
            test_cases: vec![], // TODO: Parse individual test cases
            coverage: None,     // TODO: Parse coverage if available
            raw_output: output.to_string(),
            errors: vec![],
        })
    }

    /// Parse Go test output
    fn parse_go_output(&self, output: &str, duration: Duration, success: bool) -> Result<TestResult> {
        // Simplified Go test parser
        let mut total_tests = 0;
        let mut passed = 0;
        let mut failed = 0;

        for line in output.lines() {
            if line.starts_with("PASS") || line.starts_with("FAIL") {
                total_tests += 1;
                if line.starts_with("PASS") {
                    passed += 1;
                } else {
                    failed += 1;
                }
            }
        }

        Ok(TestResult {
            success,
            total_tests,
            passed,
            failed,
            skipped: 0,
            duration,
            test_cases: vec![], // TODO: Parse individual test cases
            coverage: None,     // TODO: Parse coverage if available
            raw_output: output.to_string(),
            errors: vec![],
        })
    }
}

/// Default test framework configurations
pub fn default_test_frameworks() -> Vec<TestFrameworkConfig> {
    vec![
        // Rust - Cargo
        TestFrameworkConfig {
            language: "rust".to_string(),
            framework: "cargo".to_string(),
            command: "cargo".to_string(),
            args: vec!["test".to_string()],
            detection_files: vec!["Cargo.toml".to_string()],
            env: HashMap::new(),
        },
        // JavaScript/TypeScript - Jest
        TestFrameworkConfig {
            language: "javascript".to_string(),
            framework: "jest".to_string(),
            command: "npm".to_string(),
            args: vec!["test".to_string()],
            detection_files: vec!["package.json".to_string(), "jest.config.js".to_string()],
            env: HashMap::new(),
        },
        // Python - pytest
        TestFrameworkConfig {
            language: "python".to_string(),
            framework: "pytest".to_string(),
            command: "pytest".to_string(),
            args: vec![],
            detection_files: vec!["pytest.ini".to_string(), "pyproject.toml".to_string(), "setup.cfg".to_string()],
            env: HashMap::new(),
        },
        // Go
        TestFrameworkConfig {
            language: "go".to_string(),
            framework: "go".to_string(),
            command: "go".to_string(),
            args: vec!["test".to_string(), "./...".to_string()],
            detection_files: vec!["go.mod".to_string()],
            env: HashMap::new(),
        },
        // Java - Maven
        TestFrameworkConfig {
            language: "java".to_string(),
            framework: "maven".to_string(),
            command: "mvn".to_string(),
            args: vec!["test".to_string()],
            detection_files: vec!["pom.xml".to_string()],
            env: HashMap::new(),
        },
        // Java - Gradle
        TestFrameworkConfig {
            language: "java".to_string(),
            framework: "gradle".to_string(),
            command: "gradle".to_string(),
            args: vec!["test".to_string()],
            detection_files: vec!["build.gradle".to_string(), "build.gradle.kts".to_string()],
            env: HashMap::new(),
        },
    ]
}
