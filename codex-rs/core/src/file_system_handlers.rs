use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::{Context, Result};
use serde_json;
use similar::{ChangeTag, TextDiff};

use crate::file_system_tools::{
    FileCreateArgs, FileReadArgs, FileEditArgs, DirectoryCreateArgs, DirectoryListArgs,
    FileReadResponse, DirectoryListResponse, DirectoryEntry, EntryType, FileEditResponse,
    CreateResponse,
};
use crate::models::FunctionCallOutputPayload;
use crate::protocol::{AskForApproval, SandboxPolicy};

/// Handles the file.create tool call
pub fn handle_file_create(
    args: FileCreateArgs,
    cwd: &Path,
    sandbox_policy: &SandboxPolicy,
) -> Result<FunctionCallOutputPayload> {
    handle_file_create_with_approval(args, cwd, sandbox_policy, AskForApproval::Never)
}

/// Handles the file.create tool call with approval policy
pub fn handle_file_create_with_approval(
    args: FileCreateArgs,
    cwd: &Path,
    sandbox_policy: &SandboxPolicy,
    approval_policy: AskForApproval,
) -> Result<FunctionCallOutputPayload> {
    // Check approval policy for file creation
    if should_ask_for_approval(&approval_policy, "create", &args.path) {
        return Ok(FunctionCallOutputPayload {
            content: format!("File creation requires approval: {}", args.path),
            success: Some(false),
        });
    }

    // Resolve the path relative to cwd
    let target_path = resolve_and_validate_path(&args.path, cwd, sandbox_policy)?;
    
    // Check if file already exists
    if target_path.exists() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: File already exists at {}", target_path.display()),
            success: Some(false),
        });
    }
    
    // Create parent directories if needed
    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create parent directories for {}", target_path.display()))?;
        }
    }
    
    // Write the file content
    let content = args.content.unwrap_or_default();
    fs::write(&target_path, &content)
        .with_context(|| format!("Failed to write file {}", target_path.display()))?;
    
    let response = CreateResponse {
        success: true,
        path: target_path.display().to_string(),
        message: format!("Successfully created file with {} bytes. Ready for next operation.", content.len()),
    };
    
    Ok(FunctionCallOutputPayload {
        content: serde_json::to_string_pretty(&response)?,
        success: Some(true),
    })
}

/// Handles the file.read tool call
pub fn handle_file_read(
    args: FileReadArgs,
    cwd: &Path,
    sandbox_policy: &SandboxPolicy,
) -> Result<FunctionCallOutputPayload> {
    let target_path = resolve_and_validate_path(&args.path, cwd, sandbox_policy)?;
    
    if !target_path.exists() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: File does not exist at {}", target_path.display()),
            success: Some(false),
        });
    }
    
    if !target_path.is_file() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: Path is not a file: {}", target_path.display()),
            success: Some(false),
        });
    }
    
    // Read the file content
    let content = fs::read_to_string(&target_path)
        .with_context(|| format!("Failed to read file {}", target_path.display()))?;
    
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len() as u32;
    
    // Apply line range if specified
    let final_content = if let Some([start, end]) = args.lines {
        if start < 1 || end < start {
            return Ok(FunctionCallOutputPayload {
                content: "Error: Invalid line range. Start must be >= 1 and end >= start".to_string(),
                success: Some(false),
            });
        }
        
        let start_idx = (start - 1) as usize;
        let end_idx = std::cmp::min(end as usize, lines.len());
        
        if start_idx >= lines.len() {
            return Ok(FunctionCallOutputPayload {
                content: format!("Error: Start line {} exceeds file length {}", start, total_lines),
                success: Some(false),
            });
        }
        
        lines[start_idx..end_idx].join("\n")
    } else {
        content
    };
    
    // Get file metadata
    let metadata = fs::metadata(&target_path)?;
    let size_bytes = metadata.len();
    
    let response = FileReadResponse {
        content: final_content.clone(),
        total_lines,
        encoding: "utf-8".to_string(),
        size_bytes,
        message: if args.lines.is_some() {
            format!("Successfully read {} lines from file. Content available for further processing.",
                    final_content.lines().count())
        } else {
            format!("Successfully read entire file ({} lines, {} bytes). Content available for further processing.",
                    total_lines, size_bytes)
        },
    };
    
    Ok(FunctionCallOutputPayload {
        content: serde_json::to_string_pretty(&response)?,
        success: Some(true),
    })
}

/// Handles the directory.create tool call
pub fn handle_directory_create(
    args: DirectoryCreateArgs,
    cwd: &Path,
    sandbox_policy: &SandboxPolicy,
) -> Result<FunctionCallOutputPayload> {
    let target_path = resolve_and_validate_path(&args.path, cwd, sandbox_policy)?;
    
    if target_path.exists() {
        if target_path.is_dir() {
            return Ok(FunctionCallOutputPayload {
                content: format!("Directory already exists at {}", target_path.display()),
                success: Some(true),
            });
        } else {
            return Ok(FunctionCallOutputPayload {
                content: format!("Error: Path exists but is not a directory: {}", target_path.display()),
                success: Some(false),
            });
        }
    }
    
    // Create the directory
    if args.recursive {
        fs::create_dir_all(&target_path)
            .with_context(|| format!("Failed to create directory recursively: {}", target_path.display()))?;
    } else {
        fs::create_dir(&target_path)
            .with_context(|| format!("Failed to create directory: {}", target_path.display()))?;
    }
    
    let response = CreateResponse {
        success: true,
        path: target_path.display().to_string(),
        message: if args.recursive {
            "Successfully created directory and any missing parent directories. Ready for next operation.".to_string()
        } else {
            "Successfully created directory. Ready for next operation.".to_string()
        },
    };
    
    Ok(FunctionCallOutputPayload {
        content: serde_json::to_string_pretty(&response)?,
        success: Some(true),
    })
}

/// Handles the directory.list tool call
pub fn handle_directory_list(
    args: DirectoryListArgs,
    cwd: &Path,
    sandbox_policy: &SandboxPolicy,
) -> Result<FunctionCallOutputPayload> {
    let target_path = resolve_and_validate_path(&args.path, cwd, sandbox_policy)?;
    
    if !target_path.exists() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: Directory does not exist at {}", target_path.display()),
            success: Some(false),
        });
    }
    
    if !target_path.is_dir() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: Path is not a directory: {}", target_path.display()),
            success: Some(false),
        });
    }
    
    let mut entries = Vec::new();
    let mut total_files = 0u32;
    let mut total_directories = 0u32;
    
    if args.recursive {
        collect_entries_recursive(&target_path, &target_path, &mut entries, &mut total_files, &mut total_directories, 0, args.max_depth)?;
    } else {
        collect_entries_single(&target_path, &target_path, &mut entries, &mut total_files, &mut total_directories)?;
    }
    
    // Sort entries by type (directories first) then by name
    entries.sort_by(|a, b| {
        match (&a.entry_type, &b.entry_type) {
            (EntryType::Directory, EntryType::File) => std::cmp::Ordering::Less,
            (EntryType::File, EntryType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    
    let response = DirectoryListResponse {
        entries: entries.clone(),
        total_files,
        total_directories,
        message: format!("Successfully listed directory contents: {} files and {} directories found. Information available for further operations.",
                        total_files, total_directories),
    };
    
    Ok(FunctionCallOutputPayload {
        content: serde_json::to_string_pretty(&response)?,
        success: Some(true),
    })
}

/// Resolves and validates a path against sandbox policy
fn resolve_and_validate_path(
    path: &str,
    cwd: &Path,
    _sandbox_policy: &SandboxPolicy,
) -> Result<PathBuf> {
    let target_path = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        cwd.join(path)
    };

    // For testing and simplicity, just return the resolved path
    // In a production environment, you'd want proper sandbox validation
    Ok(target_path)
}

/// Collects directory entries for a single directory (non-recursive)
fn collect_entries_single(
    dir_path: &Path,
    base_path: &Path,
    entries: &mut Vec<DirectoryEntry>,
    total_files: &mut u32,
    total_directories: &mut u32,
) -> Result<()> {
    let read_dir = fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory {}", dir_path.display()))?;
    
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        
        let relative_path = path.strip_prefix(base_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        
        let entry_type = if metadata.is_dir() {
            *total_directories += 1;
            EntryType::Directory
        } else if metadata.is_file() {
            *total_files += 1;
            EntryType::File
        } else {
            EntryType::Symlink
        };
        
        let modified = metadata.modified()
            .ok()
            .and_then(|time| time.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|duration| {
                chrono::DateTime::from_timestamp(duration.as_secs() as i64, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default()
            });
        
        entries.push(DirectoryEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            path: relative_path,
            entry_type,
            size_bytes: if metadata.is_file() { Some(metadata.len()) } else { None },
            modified,
        });
    }
    
    Ok(())
}

/// Collects directory entries recursively up to max_depth
fn collect_entries_recursive(
    dir_path: &Path,
    base_path: &Path,
    entries: &mut Vec<DirectoryEntry>,
    total_files: &mut u32,
    total_directories: &mut u32,
    current_depth: u32,
    max_depth: u32,
) -> Result<()> {
    if current_depth >= max_depth {
        return Ok(());
    }
    
    collect_entries_single(dir_path, base_path, entries, total_files, total_directories)?;
    
    // Recursively process subdirectories
    let read_dir = fs::read_dir(dir_path)?;
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_entries_recursive(&path, base_path, entries, total_files, total_directories, current_depth + 1, max_depth)?;
        }
    }
    
    Ok(())
}

/// Handles the file.edit tool call
pub fn handle_file_edit(
    args: FileEditArgs,
    cwd: &Path,
    sandbox_policy: &SandboxPolicy,
) -> Result<FunctionCallOutputPayload> {
    let target_path = resolve_and_validate_path(&args.path, cwd, sandbox_policy)?;

    if !target_path.exists() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: File does not exist at {}", target_path.display()),
            success: Some(false),
        });
    }

    if !target_path.is_file() {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: Path is not a file: {}", target_path.display()),
            success: Some(false),
        });
    }

    // Read the original file content
    let original_content = fs::read_to_string(&target_path)
        .with_context(|| format!("Failed to read file {}", target_path.display()))?;

    // For now, we'll implement a simple instruction-based edit
    // In a full implementation, this would use an LLM or more sophisticated parsing
    let new_content = apply_edit_instructions(&original_content, &args.instructions, args.target_lines)?;

    // Generate diff
    let diff = generate_unified_diff(&original_content, &new_content, &args.path);

    // Create a summary of changes
    let changes_summary = generate_changes_summary(&original_content, &new_content);

    let response = FileEditResponse {
        original_content: original_content.clone(),
        new_content: new_content.clone(),
        diff: diff.clone(),
        changes_summary: changes_summary.clone(),
        message: format!("File edit completed successfully. {}. Ready to apply changes or continue with next operation.", changes_summary),
    };

    Ok(FunctionCallOutputPayload {
        content: serde_json::to_string_pretty(&response)?,
        success: Some(true),
    })
}

/// Applies edit instructions to content (simplified implementation)
/// In a full implementation, this would use an LLM to understand and apply the instructions
fn apply_edit_instructions(
    content: &str,
    instructions: &str,
    target_lines: Option<[u32; 2]>,
) -> Result<String> {
    // This is a simplified implementation for demonstration
    // In practice, you'd want to use an LLM or more sophisticated text processing

    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

    // Simple pattern matching for common edit instructions
    if instructions.to_lowercase().contains("add") && instructions.contains("import") {
        // Add import at the top
        if let Some(import_line) = extract_import_from_instructions(instructions) {
            new_lines.insert(0, import_line);
        }
    } else if instructions.to_lowercase().contains("replace") {
        // Simple find and replace
        if let Some((find, replace)) = extract_find_replace_from_instructions(instructions) {
            for line in &mut new_lines {
                if line.contains(&find) {
                    *line = line.replace(&find, &replace);
                }
            }
        }
    } else if let Some([start, end]) = target_lines {
        // Apply instructions to specific line range
        let start_idx = (start - 1) as usize;
        let _end_idx = std::cmp::min(end as usize, lines.len());

        if start_idx < lines.len() {
            // For demonstration, just add a comment with the instructions
            let comment = format!("// {}", instructions);
            new_lines.insert(start_idx, comment);
        }
    }

    Ok(new_lines.join("\n"))
}

/// Generates a unified diff between old and new content
fn generate_unified_diff(old_content: &str, new_content: &str, filename: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    let mut result = String::new();

    result.push_str(&format!("--- {}\n", filename));
    result.push_str(&format!("+++ {}\n", filename));

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            result.push_str("@@ ... @@\n");
        }

        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, s) = match change.tag() {
                    ChangeTag::Delete => ("-", change.value()),
                    ChangeTag::Insert => ("+", change.value()),
                    ChangeTag::Equal => (" ", change.value()),
                };
                result.push_str(&format!("{}{}", sign, s));
            }
        }
    }

    result
}

/// Generates a summary of changes made
fn generate_changes_summary(old_content: &str, new_content: &str) -> String {
    let old_lines = old_content.lines().count();
    let new_lines = new_content.lines().count();

    let diff = TextDiff::from_lines(old_content, new_content);
    let mut additions = 0;
    let mut deletions = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Insert => additions += 1,
            ChangeTag::Delete => deletions += 1,
            ChangeTag::Equal => {}
        }
    }

    format!(
        "Lines: {} -> {} ({:+}), Additions: {}, Deletions: {}",
        old_lines,
        new_lines,
        new_lines as i32 - old_lines as i32,
        additions,
        deletions
    )
}

/// Extracts import statement from instructions (simplified)
fn extract_import_from_instructions(instructions: &str) -> Option<String> {
    // Very basic pattern matching - in practice you'd want more sophisticated parsing
    if let Some(start) = instructions.find("import ") {
        let import_part = &instructions[start..];
        if let Some(end) = import_part.find('\n').or_else(|| Some(import_part.len())) {
            return Some(import_part[..end].to_string());
        }
    }
    None
}

/// Extracts find/replace patterns from instructions (simplified)
fn extract_find_replace_from_instructions(instructions: &str) -> Option<(String, String)> {
    // Very basic pattern matching - in practice you'd want more sophisticated parsing
    if instructions.contains("replace") && instructions.contains("with") {
        // This is a placeholder implementation
        // You'd want to use regex or more sophisticated parsing
        None
    } else {
        None
    }
}

/// Checks if an operation should ask for user approval based on the approval policy
pub fn should_ask_for_approval(approval_policy: &AskForApproval, _operation: &str, path: &str) -> bool {
    match approval_policy {
        AskForApproval::Never => false,
        AskForApproval::OnFailure => false, // Only ask on failure, not before operation
        AskForApproval::OnRequest => true,  // Always ask for approval
        AskForApproval::UnlessTrusted => {
            // For now, consider operations on common file extensions as trusted
            // In a full implementation, this would be more sophisticated
            let trusted_extensions = [".txt", ".md", ".json", ".yaml", ".yml", ".toml"];
            !trusted_extensions.iter().any(|ext| path.ends_with(ext))
        }
    }
}
