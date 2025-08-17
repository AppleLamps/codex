//! Integration tests for LSP and test framework functionality.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp_client::{LspServerConfig, default_lsp_configs};
    use crate::lsp_handlers::LspManager;
    use crate::test_runner::{TestRunner, default_test_frameworks};
    use crate::test_handlers::TestManager;
    use crate::code_analysis::{CodeAnalyzer, default_analysis_config};
    use std::path::PathBuf;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_lsp_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();
        
        let lsp_manager = LspManager::new(workspace_root.clone());
        assert_eq!(lsp_manager.workspace_root(), workspace_root);
    }

    #[tokio::test]
    async fn test_test_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();
        
        let test_manager = TestManager::new(workspace_root.clone());
        assert_eq!(test_manager.workspace_root(), workspace_root);
    }

    #[tokio::test]
    async fn test_default_lsp_configs() {
        let configs = default_lsp_configs();
        assert!(!configs.is_empty());
        
        // Check that we have configs for common languages
        let languages: Vec<&str> = configs.iter().map(|c| c.language.as_str()).collect();
        assert!(languages.contains(&"rust"));
        assert!(languages.contains(&"typescript"));
        assert!(languages.contains(&"python"));
    }

    #[tokio::test]
    async fn test_default_test_frameworks() {
        let frameworks = default_test_frameworks();
        assert!(!frameworks.is_empty());
        
        // Check that we have frameworks for common languages
        let framework_names: Vec<&str> = frameworks.iter().map(|f| f.framework.as_str()).collect();
        assert!(framework_names.contains(&"cargo"));
        assert!(framework_names.contains(&"jest"));
        assert!(framework_names.contains(&"pytest"));
    }

    #[tokio::test]
    async fn test_test_runner_framework_detection() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();
        
        // Create a Cargo.toml file to simulate a Rust project
        let cargo_toml = workspace_root.join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"test\"\nversion = \"0.1.0\"").await.unwrap();
        
        let test_runner = TestRunner::new(workspace_root);
        let detected_framework = test_runner.detect_framework();
        
        assert!(detected_framework.is_some());
        assert_eq!(detected_framework.unwrap().framework, "cargo");
    }

    #[tokio::test]
    async fn test_code_analyzer_creation() {
        let config = default_analysis_config();
        let analyzer = CodeAnalyzer::new(config);
        
        // Test that the analyzer was created successfully
        // (This is a simple test since CodeAnalyzer doesn't expose much publicly)
    }

    #[tokio::test]
    async fn test_code_analysis_on_sample_file() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();
        
        // Create a sample Rust file
        let rust_file = workspace_root.join("sample.rs");
        let rust_content = r#"
// This is a sample Rust file for testing
fn main() {
    println!("Hello, world!");
    if true {
        println!("This is a conditional");
    }
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

struct TestStruct {
    field: String,
}
"#;
        fs::write(&rust_file, rust_content).await.unwrap();
        
        let config = default_analysis_config();
        let analyzer = CodeAnalyzer::new(config);
        
        let result = analyzer.analyze_file(&rust_file).await.unwrap();
        
        assert_eq!(result.language, "rust");
        assert!(result.metrics.lines_of_code > 0);
        assert!(result.metrics.function_count >= 2); // main and add functions
        assert!(result.quality_score > 0.0);
    }

    #[tokio::test]
    async fn test_lsp_tools_creation() {
        use crate::lsp_tools::*;
        
        // Test that all LSP tools can be created
        let complete_tool = create_code_complete_tool();
        let diagnostics_tool = create_code_diagnostics_tool();
        let definition_tool = create_code_definition_tool();
        let references_tool = create_code_references_tool();
        let symbols_tool = create_code_symbols_tool();
        let hover_tool = create_code_hover_tool();
        
        // Verify tool names
        if let crate::openai_tools::OpenAiTool::Function(tool) = complete_tool {
            assert_eq!(tool.name, "code.complete");
        }
        
        if let crate::openai_tools::OpenAiTool::Function(tool) = diagnostics_tool {
            assert_eq!(tool.name, "code.diagnostics");
        }
        
        if let crate::openai_tools::OpenAiTool::Function(tool) = definition_tool {
            assert_eq!(tool.name, "code.definition");
        }
    }

    #[tokio::test]
    async fn test_test_tools_creation() {
        use crate::test_tools::*;
        
        // Test that all test tools can be created
        let run_tool = create_test_run_tool();
        let analyze_tool = create_test_analyze_tool();
        let generate_tool = create_test_generate_tool();
        let debug_tool = create_test_debug_tool();
        
        // Verify tool names
        if let crate::openai_tools::OpenAiTool::Function(tool) = run_tool {
            assert_eq!(tool.name, "test.run");
        }
        
        if let crate::openai_tools::OpenAiTool::Function(tool) = analyze_tool {
            assert_eq!(tool.name, "test.analyze");
        }
        
        if let crate::openai_tools::OpenAiTool::Function(tool) = generate_tool {
            assert_eq!(tool.name, "test.generate");
        }
        
        if let crate::openai_tools::OpenAiTool::Function(tool) = debug_tool {
            assert_eq!(tool.name, "test.debug");
        }
    }

    #[tokio::test]
    async fn test_tools_config_includes_new_tools() {
        use crate::openai_tools::ToolsConfig;
        use crate::model_family::find_family_for_model;
        use crate::protocol::{AskForApproval, SandboxPolicy};
        
        let model_family = find_family_for_model("gpt-4o").unwrap();
        let config = ToolsConfig::new(
            &model_family,
            AskForApproval::Never,
            SandboxPolicy::ReadOnly,
            false, // plan_tool
            false, // apply_patch_tool
        );
        
        // Verify that LSP and test tools are enabled by default
        assert!(config.lsp_tools);
        assert!(config.test_tools);
    }

    #[tokio::test]
    async fn test_openai_tools_includes_new_tools() {
        use crate::openai_tools::{ToolsConfig, get_openai_tools};
        use crate::model_family::find_family_for_model;
        use crate::protocol::{AskForApproval, SandboxPolicy};
        
        let model_family = find_family_for_model("gpt-4o").unwrap();
        let config = ToolsConfig::new(
            &model_family,
            AskForApproval::Never,
            SandboxPolicy::ReadOnly,
            false, // plan_tool
            false, // apply_patch_tool
        );
        
        let tools = get_openai_tools(&config, None, None);
        
        // Count LSP and test tools
        let lsp_tool_count = tools.iter().filter(|tool| {
            if let crate::openai_tools::OpenAiTool::Function(f) = tool {
                f.name.starts_with("code.")
            } else {
                false
            }
        }).count();
        
        let test_tool_count = tools.iter().filter(|tool| {
            if let crate::openai_tools::OpenAiTool::Function(f) = tool {
                f.name.starts_with("test.")
            } else {
                false
            }
        }).count();
        
        // We should have 6 LSP tools and 4 test tools
        assert_eq!(lsp_tool_count, 6);
        assert_eq!(test_tool_count, 4);
    }

    #[tokio::test]
    async fn test_language_detection() {
        use crate::code_analysis::detect_language;
        use std::path::Path;
        
        assert_eq!(detect_language(Path::new("test.rs")), "rust");
        assert_eq!(detect_language(Path::new("test.js")), "javascript");
        assert_eq!(detect_language(Path::new("test.ts")), "typescript");
        assert_eq!(detect_language(Path::new("test.py")), "python");
        assert_eq!(detect_language(Path::new("test.go")), "go");
        assert_eq!(detect_language(Path::new("test.java")), "java");
    }

    #[tokio::test]
    async fn test_file_extension_support() {
        use crate::code_analysis::get_supported_extensions;
        
        let extensions = get_supported_extensions();
        assert!(extensions.contains(&"rs"));
        assert!(extensions.contains(&"js"));
        assert!(extensions.contains(&"ts"));
        assert!(extensions.contains(&"py"));
        assert!(extensions.contains(&"go"));
        assert!(extensions.contains(&"java"));
    }

    #[tokio::test]
    async fn test_code_metrics_calculation() {
        use crate::code_analysis::analyze_metrics;
        
        let sample_code = r#"
// This is a comment
fn main() {
    println!("Hello");
    if true {
        println!("World");
    }
}

fn test() {
    // Another comment
}
"#;
        
        let metrics = analyze_metrics(sample_code, "rust");
        
        assert!(metrics.lines_of_code > 0);
        assert!(metrics.lines_of_comments > 0);
        assert!(metrics.function_count >= 2);
        assert!(metrics.cyclomatic_complexity > 1);
    }

    #[tokio::test]
    async fn test_issue_detection() {
        use crate::code_analysis::analyze_issues;
        use std::path::Path;
        
        let sample_code = r#"
fn main() {
    let result = some_function().unwrap(); // This should trigger an issue
    println!("This line is way too long and should trigger a line length issue because it exceeds the recommended maximum line length");
    // TODO: Fix this later
}
"#;
        
        let issues = analyze_issues(sample_code, "rust", Path::new("test.rs"));
        
        // Should detect unwrap usage, long line, and TODO comment
        assert!(issues.len() >= 3);
        
        let has_unwrap_issue = issues.iter().any(|issue| issue.message.contains("unwrap"));
        let has_long_line_issue = issues.iter().any(|issue| issue.message.contains("Line too long"));
        let has_todo_issue = issues.iter().any(|issue| issue.message.contains("TODO"));
        
        assert!(has_unwrap_issue);
        assert!(has_long_line_issue);
        assert!(has_todo_issue);
    }

    #[tokio::test]
    async fn test_openrouter_provider_integration() {
        use crate::model_provider_info::built_in_model_providers;

        let providers = built_in_model_providers();
        let openrouter = providers.get("openrouter").expect("OpenRouter should be available");

        assert_eq!(openrouter.name, "OpenRouter");
        assert_eq!(openrouter.base_url, Some("https://openrouter.ai/api/v1".to_string()));
        assert_eq!(openrouter.env_key, Some("OPENROUTER_API_KEY".to_string()));
    }

    #[tokio::test]
    async fn test_openrouter_model_families_integration() {
        use crate::model_family::find_family_for_model;

        // Test all OpenRouter models are recognized
        let models = [
            "openai/gpt-5",
            "anthropic/claude-sonnet-4",
            "moonshotai/kimi-k2:free",
            "qwen/qwen3-coder:free"
        ];

        for model in models {
            let family = find_family_for_model(model);
            assert!(family.is_some(), "Model {} should be recognized", model);

            let family = family.unwrap();
            assert_eq!(family.slug, model);
        }
    }

    #[tokio::test]
    async fn test_openrouter_model_info_integration() {
        use crate::openai_model_info::get_model_info;
        use crate::model_family::find_family_for_model;

        // Test all OpenRouter models have correct context windows
        let expected_info = [
            ("openai/gpt-5", 400_000, 180_000),
            ("anthropic/claude-sonnet-4", 200_000, 64_000),
            ("moonshotai/kimi-k2:free", 65_500, 65_500),
            ("qwen/qwen3-coder:free", 262_144, 262_144),
        ];

        for (model, expected_context, expected_output) in expected_info {
            let family = find_family_for_model(model).expect(&format!("Model {} should be recognized", model));
            let info = get_model_info(&family);
            assert!(info.is_some(), "Model {} should have info", model);

            let info = info.unwrap();
            assert_eq!(info.context_window, expected_context, "Context window mismatch for {}", model);
            assert_eq!(info.max_output_tokens, expected_output, "Output tokens mismatch for {}", model);
        }
    }

    #[tokio::test]
    async fn test_openrouter_tools_config_compatibility() {
        use crate::openai_tools::{ToolsConfig, get_openai_tools};
        use crate::model_family::find_family_for_model;
        use crate::protocol::{AskForApproval, SandboxPolicy};

        // Test that OpenRouter models work with tools config
        let models = [
            "openai/gpt-5",
            "anthropic/claude-sonnet-4",
            "moonshotai/kimi-k2:free",
            "qwen/qwen3-coder:free"
        ];

        for model in models {
            let model_family = find_family_for_model(model).expect(&format!("Model {} should be recognized", model));
            let config = ToolsConfig::new(
                &model_family,
                AskForApproval::Never,
                SandboxPolicy::ReadOnly,
                false, // plan_tool
                false, // apply_patch_tool
            );

            let tools = get_openai_tools(&config, None, None);
            assert!(!tools.is_empty(), "Tools should be available for {}", model);

            // Verify LSP and test tools are included
            let has_lsp_tools = tools.iter().any(|tool| {
                if let crate::openai_tools::OpenAiTool::Function(f) = tool {
                    f.name.starts_with("code.")
                } else {
                    false
                }
            });

            let has_test_tools = tools.iter().any(|tool| {
                if let crate::openai_tools::OpenAiTool::Function(f) = tool {
                    f.name.starts_with("test.")
                } else {
                    false
                }
            });

            assert!(has_lsp_tools, "LSP tools should be available for {}", model);
            assert!(has_test_tools, "Test tools should be available for {}", model);
        }
    }

    #[tokio::test]
    async fn test_openrouter_reasoning_support() {
        use crate::model_family::find_family_for_model;

        // Test reasoning support for advanced models
        let gpt5_family = find_family_for_model("openai/gpt-5").unwrap();
        assert!(gpt5_family.supports_reasoning_summaries, "GPT-5 should support reasoning");

        let claude_family = find_family_for_model("anthropic/claude-sonnet-4").unwrap();
        assert!(claude_family.supports_reasoning_summaries, "Claude Sonnet 4 should support reasoning");

        // Test that free models don't claim reasoning support
        let kimi_family = find_family_for_model("moonshotai/kimi-k2:free").unwrap();
        assert!(!kimi_family.supports_reasoning_summaries, "Kimi K2 free should not claim reasoning support");

        let qwen_family = find_family_for_model("qwen/qwen3-coder:free").unwrap();
        assert!(!qwen_family.supports_reasoning_summaries, "Qwen 3 Coder free should not claim reasoning support");
    }
}
