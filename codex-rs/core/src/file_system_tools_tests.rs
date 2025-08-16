#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use crate::protocol::{AskForApproval, SandboxPolicy};
    use crate::file_system_handlers::*;
    use crate::file_system_tools::*;
    use crate::openai_tools::OpenAiTool;

    fn create_test_sandbox_policy() -> SandboxPolicy {
        SandboxPolicy::ReadOnly
    }

    #[test]
    fn test_file_create_tool_definition() {
        let tool = create_file_create_tool();
        match tool {
            OpenAiTool::Function(func) => {
                assert_eq!(func.name, "file.create");
                assert!(func.description.contains("Create a new file"));
            }
            _ => panic!("Expected Function tool"),
        }
    }

    #[test]
    fn test_file_read_tool_definition() {
        let tool = create_file_read_tool();
        match tool {
            OpenAiTool::Function(func) => {
                assert_eq!(func.name, "file.read");
                assert!(func.description.contains("Read the contents"));
            }
            _ => panic!("Expected Function tool"),
        }
    }

    #[test]
    fn test_file_edit_tool_definition() {
        let tool = create_file_edit_tool();
        match tool {
            OpenAiTool::Function(func) => {
                assert_eq!(func.name, "file.edit");
                assert!(func.description.contains("Edit a file"));
            }
            _ => panic!("Expected Function tool"),
        }
    }

    #[test]
    fn test_directory_create_tool_definition() {
        let tool = create_directory_create_tool();
        match tool {
            OpenAiTool::Function(func) => {
                assert_eq!(func.name, "directory.create");
                assert!(func.description.contains("Create a new directory"));
            }
            _ => panic!("Expected Function tool"),
        }
    }

    #[test]
    fn test_directory_list_tool_definition() {
        let tool = create_directory_list_tool();
        match tool {
            OpenAiTool::Function(func) => {
                assert_eq!(func.name, "directory.list");
                assert!(func.description.contains("List directory contents"));
            }
            _ => panic!("Expected Function tool"),
        }
    }

    #[test]
    fn test_handle_file_create_success() {
        let temp_dir = TempDir::new().unwrap();
        let args = FileCreateArgs {
            path: "test.txt".to_string(),
            content: Some("Hello, World!".to_string()),
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_file_create(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(true));
        
        // Verify file was created
        let file_path = temp_dir.path().join("test.txt");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_handle_file_create_empty_content() {
        let temp_dir = TempDir::new().unwrap();
        let args = FileCreateArgs {
            path: "empty.txt".to_string(),
            content: None,
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_file_create(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let file_path = temp_dir.path().join("empty.txt");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "");
    }

    #[test]
    fn test_handle_file_create_with_subdirectory() {
        let temp_dir = TempDir::new().unwrap();
        let args = FileCreateArgs {
            path: "subdir/test.txt".to_string(),
            content: Some("Content".to_string()),
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_file_create(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let file_path = temp_dir.path().join("subdir/test.txt");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Content");
    }

    #[test]
    fn test_handle_file_create_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("existing.txt");
        fs::write(&file_path, "existing content").unwrap();

        let args = FileCreateArgs {
            path: "existing.txt".to_string(),
            content: Some("new content".to_string()),
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_file_create(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(false));
        assert!(output.content.contains("already exists"));
    }

    #[test]
    fn test_handle_file_read_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Line 1\nLine 2\nLine 3\n";
        fs::write(&file_path, content).unwrap();

        let args = FileReadArgs {
            path: "test.txt".to_string(),
            lines: None,
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_file_read(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(true));
        
        let response: FileReadResponse = serde_json::from_str(&output.content).unwrap();
        assert_eq!(response.content, content);
        assert_eq!(response.total_lines, 3);
        assert!(response.message.contains("Successfully read entire file"));
    }

    #[test]
    fn test_handle_file_read_with_line_range() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Line 1\nLine 2\nLine 3\nLine 4\n";
        fs::write(&file_path, content).unwrap();

        let args = FileReadArgs {
            path: "test.txt".to_string(),
            lines: Some([2, 3]),
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_file_read(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(true));
        
        let response: FileReadResponse = serde_json::from_str(&output.content).unwrap();
        assert_eq!(response.content, "Line 2\nLine 3");
        assert_eq!(response.total_lines, 4);
        assert!(response.message.contains("Successfully read"));
    }

    #[test]
    fn test_handle_file_read_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let args = FileReadArgs {
            path: "nonexistent.txt".to_string(),
            lines: None,
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_file_read(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(false));
        assert!(output.content.contains("does not exist"));
    }

    #[test]
    fn test_handle_directory_create_success() {
        let temp_dir = TempDir::new().unwrap();
        let args = DirectoryCreateArgs {
            path: "new_dir".to_string(),
            recursive: false,
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_directory_create(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(true));
        
        let dir_path = temp_dir.path().join("new_dir");
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_handle_directory_create_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let args = DirectoryCreateArgs {
            path: "parent/child/grandchild".to_string(),
            recursive: true,
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_directory_create(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(true));
        
        let dir_path = temp_dir.path().join("parent/child/grandchild");
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());
    }

    #[test]
    fn test_handle_directory_list_success() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test files and directories
        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).unwrap();

        let args = DirectoryListArgs {
            path: ".".to_string(),
            recursive: false,
            max_depth: 2,
        };
        let sandbox_policy = create_test_sandbox_policy();

        let result = handle_directory_list(args, temp_dir.path(), &sandbox_policy);
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert_eq!(output.success, Some(true));
        
        let response: DirectoryListResponse = serde_json::from_str(&output.content).unwrap();
        assert_eq!(response.total_files, 2);
        assert_eq!(response.total_directories, 1);
        assert_eq!(response.entries.len(), 3);
        assert!(response.message.contains("Successfully listed directory contents"));
    }

    #[test]
    fn test_approval_policy_never() {
        assert!(!should_ask_for_approval(&AskForApproval::Never, "create", "test.txt"));
    }

    #[test]
    fn test_approval_policy_on_request() {
        assert!(should_ask_for_approval(&AskForApproval::OnRequest, "create", "test.txt"));
    }

    #[test]
    fn test_approval_policy_unless_trusted() {
        // Trusted extensions should not require approval
        assert!(!should_ask_for_approval(&AskForApproval::UnlessTrusted, "create", "test.txt"));
        assert!(!should_ask_for_approval(&AskForApproval::UnlessTrusted, "create", "config.json"));

        // Non-trusted extensions should require approval
        assert!(should_ask_for_approval(&AskForApproval::UnlessTrusted, "create", "script.sh"));
        assert!(should_ask_for_approval(&AskForApproval::UnlessTrusted, "create", "binary.exe"));
    }
}
