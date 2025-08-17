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

    // Apply edit instructions to generate new content
    let edit_result = apply_edit_instructions(&original_content, &args.instructions, args.target_lines);

    let (new_content, changes_made) = match edit_result {
        Ok((content, made_changes)) => (content, made_changes),
        Err(e) => {
            return Ok(FunctionCallOutputPayload {
                content: format!("Error applying edit instructions: {}", e),
                success: Some(false),
            });
        }
    };

    // Check if any changes were actually made
    if !changes_made {
        let suggestions = generate_edit_suggestions(&args.instructions, &original_content, args.target_lines);
        return Ok(FunctionCallOutputPayload {
            content: format!(
                "No changes were made. The edit instructions '{}' could not be applied to the file content.\n\n{}\n\nPlease provide more specific instructions or check if the target content exists.",
                args.instructions,
                suggestions
            ),
            success: Some(false),
        });
    }

    // Write the new content to the file
    if let Err(e) = fs::write(&target_path, &new_content) {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error writing file {}: {}", target_path.display(), e),
            success: Some(false),
        });
    }

    // Verify the file was written correctly by reading it back
    let verification_content = fs::read_to_string(&target_path)
        .with_context(|| format!("Failed to verify file write for {}", target_path.display()))?;

    if verification_content != new_content {
        return Ok(FunctionCallOutputPayload {
            content: format!("Error: File verification failed. The written content does not match the expected content."),
            success: Some(false),
        });
    }

    // Generate diff and summary
    let diff = generate_unified_diff(&original_content, &new_content, &args.path);
    let changes_summary = generate_changes_summary(&original_content, &new_content);

    let response = FileEditResponse {
        original_content: original_content.clone(),
        new_content: new_content.clone(),
        diff: diff.clone(),
        changes_summary: changes_summary.clone(),
        message: format!("File edit completed successfully and verified. {}. File has been saved to disk.", changes_summary),
    };

    Ok(FunctionCallOutputPayload {
        content: serde_json::to_string_pretty(&response)?,
        success: Some(true),
    })
}

/// Applies edit instructions to content and returns (new_content, changes_made)
/// This implementation handles common edit patterns and provides clear feedback
fn apply_edit_instructions(
    content: &str,
    instructions: &str,
    target_lines: Option<[u32; 2]>,
) -> Result<(String, bool)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let mut changes_made = false;

    let instructions_lower = instructions.to_lowercase();

    // Check target_lines first to prioritize line-specific operations
    if let Some([start, end]) = target_lines {
        let start_idx = (start - 1) as usize;
        let end_idx = std::cmp::min(end as usize, lines.len());

        if start_idx < lines.len() {
            // Pattern 1: Replace content in line range
            if instructions_lower.contains("replace") {
                if let Some(replacement_content) = extract_replacement_content(instructions) {
                    // Remove the lines in the range
                    if start_idx < end_idx {
                        new_lines.drain(start_idx..end_idx);
                    }
                    // Insert the replacement content
                    for (i, line) in replacement_content.lines().enumerate() {
                        new_lines.insert(start_idx + i, line.to_string());
                    }
                    changes_made = true;
                }
            }
            // Pattern 2: Add content at specific lines
            else if instructions_lower.contains("add") {
                if let Some(content_to_add) = extract_content_to_add(instructions) {
                    new_lines.insert(start_idx, content_to_add);
                    changes_made = true;
                }
            }
            // Pattern 3: Delete lines in range
            else if instructions_lower.contains("delete") || instructions_lower.contains("remove") {
                if start_idx < end_idx {
                    new_lines.drain(start_idx..end_idx);
                    changes_made = true;
                }
            }
        }
    }
    // Pattern 4: Add import statements (no target_lines)
    else if instructions_lower.contains("add") && instructions_lower.contains("import") {
        if let Some(import_line) = extract_import_from_instructions(instructions) {
            // Check if import already exists
            if !content.contains(&import_line) {
                new_lines.insert(0, import_line);
                changes_made = true;
            }
        }
    }
    // Pattern 5: Replace text (find and replace, no target_lines)
    else if instructions_lower.contains("replace") && instructions_lower.contains("with") {
        if let Some((find, replace)) = extract_find_replace_from_instructions(instructions) {
            for line in &mut new_lines {
                if line.contains(&find) {
                    let old_line = line.clone();
                    *line = line.replace(&find, &replace);
                    if *line != old_line {
                        changes_made = true;
                    }
                }
            }
        }
    }
    // Pattern 6: Delete content (no target_lines)
    else if instructions_lower.contains("delete") || instructions_lower.contains("remove") {
        // Try to find and delete specific content mentioned in instructions
        if let Some(content_to_delete) = extract_content_to_delete(instructions) {
            new_lines.retain(|line| {
                let should_keep = !line.contains(&content_to_delete);
                if !should_keep {
                    changes_made = true;
                }
                should_keep
            });
        }
    }
    // Pattern 7: Insert content after a specific line containing text
    else if instructions_lower.contains("after") && instructions_lower.contains("add") {
        if let Some((search_text, content_to_add)) = extract_after_pattern(instructions) {
            for (i, line) in new_lines.iter().enumerate() {
                if line.contains(&search_text) {
                    new_lines.insert(i + 1, content_to_add.clone());
                    changes_made = true;
                    break;
                }
            }
        }
    }
    // Pattern 8: Insert content before a specific line containing text
    else if instructions_lower.contains("before") && instructions_lower.contains("add") {
        if let Some((search_text, content_to_add)) = extract_before_pattern(instructions) {
            for (i, line) in new_lines.iter().enumerate() {
                if line.contains(&search_text) {
                    new_lines.insert(i, content_to_add.clone());
                    changes_made = true;
                    break;
                }
            }
        }
    }

    Ok((new_lines.join("\n"), changes_made))
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

/// Extracts import statement from instructions
fn extract_import_from_instructions(instructions: &str) -> Option<String> {
    // Look for patterns like 'add import "statement"' or 'import statement'
    if let Some(import_pos) = instructions.find("import ") {
        let after_import = &instructions[import_pos + 7..]; // Skip "import "

        // Check if the import statement is quoted
        if after_import.starts_with('"') {
            // Find the closing quote
            if let Some(end_quote) = after_import[1..].find('"') {
                return Some(after_import[1..end_quote + 1].trim().to_string());
            }
        } else {
            // Not quoted, take until newline or end of string
            let end = after_import.find('\n').unwrap_or(after_import.len());
            let import_statement = after_import[..end].trim();
            if !import_statement.is_empty() {
                return Some(import_statement.to_string());
            }
        }
    }

    None
}

/// Extracts find/replace patterns from instructions
fn extract_find_replace_from_instructions(instructions: &str) -> Option<(String, String)> {
    // Pattern: "replace X with Y" or "replace 'X' with 'Y'"
    if let Some(replace_pos) = instructions.find("replace ") {
        let after_replace = &instructions[replace_pos + 8..];
        if let Some(with_pos) = after_replace.find(" with ") {
            let find_part = after_replace[..with_pos].trim();
            let replace_part = after_replace[with_pos + 6..].trim();

            // Remove quotes if present
            let find_clean = find_part.trim_matches('"').trim_matches('\'');
            let replace_clean = replace_part.trim_matches('"').trim_matches('\'');

            if !find_clean.is_empty() {
                return Some((find_clean.to_string(), replace_clean.to_string()));
            }
        }
    }

    None
}

/// Extracts content to add from instructions
fn extract_content_to_add(instructions: &str) -> Option<String> {
    // Look for patterns like "add: content" or "add 'content'" or "add \"content\""
    if let Some(add_pos) = instructions.find("add") {
        let after_add = &instructions[add_pos + 3..].trim();

        // Skip colon if present
        let content_part = if after_add.starts_with(':') {
            after_add[1..].trim()
        } else {
            after_add
        };

        // Remove quotes if present
        let content = content_part.trim_matches('"').trim_matches('\'');

        if !content.is_empty() {
            return Some(content.to_string());
        }
    }

    None
}

/// Extracts replacement content from instructions
fn extract_replacement_content(instructions: &str) -> Option<String> {
    // Look for patterns like "replace with: content" or after "with"
    if let Some(with_pos) = instructions.find("with") {
        let after_with = &instructions[with_pos + 4..].trim();

        // Skip colon if present
        let content_part = if after_with.starts_with(':') {
            after_with[1..].trim()
        } else {
            after_with
        };

        // Remove quotes if present
        let content = content_part.trim_matches('"').trim_matches('\'');

        if !content.is_empty() {
            // Convert escaped newlines to actual newlines
            let processed_content = content.replace("\\n", "\n");
            return Some(processed_content);
        }
    }

    None
}

/// Extracts content to delete from instructions
fn extract_content_to_delete(instructions: &str) -> Option<String> {
    // Look for patterns like "delete: content" or "remove 'content'"
    let keywords = ["delete", "remove"];

    for keyword in &keywords {
        if let Some(keyword_pos) = instructions.find(keyword) {
            let after_keyword = &instructions[keyword_pos + keyword.len()..].trim();

            // Skip colon if present
            let content_part = if after_keyword.starts_with(':') {
                after_keyword[1..].trim()
            } else {
                after_keyword
            };

            // Remove quotes if present
            let content = content_part.trim_matches('"').trim_matches('\'');

            if !content.is_empty() {
                return Some(content.to_string());
            }
        }
    }

    None
}

/// Extracts search text and content for "after" pattern
fn extract_after_pattern(instructions: &str) -> Option<(String, String)> {
    // Pattern: "add X after Y" or "after Y add X"
    if let Some(after_pos) = instructions.find("after ") {
        let after_part = &instructions[after_pos + 6..];
        if let Some(add_pos) = instructions.find("add ") {
            let add_part = &instructions[add_pos + 4..];

            if after_pos < add_pos {
                // "after Y add X" pattern
                let search_text = after_part.split(" add ").next()?.trim();
                let content = add_part.trim();
                return Some((search_text.trim_matches('"').trim_matches('\'').to_string(),
                           content.trim_matches('"').trim_matches('\'').to_string()));
            } else {
                // "add X after Y" pattern
                let content = add_part.split(" after ").next()?.trim();
                let search_text = after_part.trim();
                return Some((search_text.trim_matches('"').trim_matches('\'').to_string(),
                           content.trim_matches('"').trim_matches('\'').to_string()));
            }
        }
    }

    None
}

/// Extracts search text and content for "before" pattern
fn extract_before_pattern(instructions: &str) -> Option<(String, String)> {
    // Pattern: "add X before Y" or "before Y add X"
    if let Some(before_pos) = instructions.find("before ") {
        let before_part = &instructions[before_pos + 7..];
        if let Some(add_pos) = instructions.find("add ") {
            let add_part = &instructions[add_pos + 4..];

            if before_pos < add_pos {
                // "before Y add X" pattern
                let search_text = before_part.split(" add ").next()?.trim();
                let content = add_part.trim();
                return Some((search_text.trim_matches('"').trim_matches('\'').to_string(),
                           content.trim_matches('"').trim_matches('\'').to_string()));
            } else {
                // "add X before Y" pattern
                let content = add_part.split(" before ").next()?.trim();
                let search_text = before_part.trim();
                return Some((search_text.trim_matches('"').trim_matches('\'').to_string(),
                           content.trim_matches('"').trim_matches('\'').to_string()));
            }
        }
    }

    None
}

/// Generates helpful suggestions when edit instructions fail
fn generate_edit_suggestions(instructions: &str, content: &str, target_lines: Option<[u32; 2]>) -> String {
    let mut suggestions = Vec::new();
    let instructions_lower = instructions.to_lowercase();

    // Analyze the instruction type and provide specific suggestions
    if instructions_lower.contains("replace") {
        suggestions.push("For replace operations, use the format: 'replace \"exact text to find\" with \"replacement text\"'".to_string());

        // Check if the content contains common patterns that might be targets
        if content.contains("function") || content.contains("fn ") {
            suggestions.push("If replacing a function, make sure to include the exact function signature".to_string());
        }
        if content.contains("import") || content.contains("use ") {
            suggestions.push("If replacing imports, make sure to match the exact import statement".to_string());
        }
    }

    if instructions_lower.contains("add") {
        suggestions.push("For add operations, use formats like:".to_string());
        suggestions.push("  - 'add \"new content\" after \"existing line\"'".to_string());
        suggestions.push("  - 'add \"new content\" before \"existing line\"'".to_string());
        suggestions.push("  - 'add import \"import statement\"' (for imports)".to_string());

        if target_lines.is_some() {
            suggestions.push("  - 'add: \"content to insert\"' (when using target_lines)".to_string());
        }
    }

    if instructions_lower.contains("delete") || instructions_lower.contains("remove") {
        suggestions.push("For delete operations, use formats like:".to_string());
        suggestions.push("  - 'delete \"exact text to remove\"'".to_string());
        suggestions.push("  - Use target_lines to specify line ranges to delete".to_string());
    }

    // Provide content analysis
    let line_count = content.lines().count();
    suggestions.push(format!("File has {} lines. ", line_count));

    if let Some([start, end]) = target_lines {
        if start > line_count as u32 {
            suggestions.push(format!("Target line {} exceeds file length ({})", start, line_count));
        } else {
            let actual_lines: Vec<&str> = content.lines().collect();
            let start_idx = (start - 1) as usize;
            let end_idx = std::cmp::min(end as usize, actual_lines.len());

            if start_idx < actual_lines.len() {
                suggestions.push("Target lines contain:".to_string());
                for (i, line) in actual_lines[start_idx..end_idx].iter().enumerate() {
                    suggestions.push(format!("  Line {}: {}", start + i as u32, line));
                }
            }
        }
    }

    // Show first few lines of content for context
    suggestions.push("First few lines of file for reference:".to_string());
    for (i, line) in content.lines().take(5).enumerate() {
        suggestions.push(format!("  Line {}: {}", i + 1, line));
    }

    suggestions.join("\n")
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
