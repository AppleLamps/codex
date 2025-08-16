# File System Tools

The Codex system provides granular file system tools that offer a better developer experience compared to shell commands or patch operations. These tools provide structured responses with clear continuation signals to help AI agents understand when to proceed with multi-step tasks.

## Available Tools

### 1. `file.create` - Create Files

Creates a new file with specified content.

**Parameters:**
- `path` (string): Relative or absolute path for the new file
- `content` (string): Content to write to the file

**Example:**
```json
{
  "name": "file.create",
  "arguments": {
    "path": "src/new_module.rs",
    "content": "pub fn hello() {\n    println!(\"Hello, world!\");\n}"
  }
}
```

**Response:**
```json
{
  "success": true,
  "path": "/workspace/src/new_module.rs",
  "message": "Successfully created file with 67 bytes. Ready for next operation."
}
```

### 2. `file.read` - Read Files

Reads file content with optional line range support.

**Parameters:**
- `path` (string): Path to the file to read
- `lines` (optional object): Line range specification
  - `start` (number): Starting line number (1-based)
  - `end` (number): Ending line number (1-based)

**Example:**
```json
{
  "name": "file.read",
  "arguments": {
    "path": "src/main.rs",
    "lines": { "start": 10, "end": 20 }
  }
}
```

**Response:**
```json
{
  "content": "fn main() {\n    println!(\"Hello!\");\n}",
  "total_lines": 25,
  "encoding": "utf-8",
  "size_bytes": 1024,
  "message": "Successfully read 11 lines from file. Content available for further processing."
}
```

### 3. `file.edit` - Edit Files

Edits files using natural language instructions with diff preview.

**Parameters:**
- `path` (string): Path to the file to edit
- `instructions` (string): Natural language description of changes

**Example:**
```json
{
  "name": "file.edit",
  "arguments": {
    "path": "src/main.rs",
    "instructions": "Add error handling to the main function"
  }
}
```

**Response:**
```json
{
  "original_content": "fn main() {\n    println!(\"Hello!\");\n}",
  "new_content": "fn main() -> Result<(), Box<dyn std::error::Error>> {\n    println!(\"Hello!\");\n    Ok(())\n}",
  "diff": "@@ -1,3 +1,4 @@\n-fn main() {\n+fn main() -> Result<(), Box<dyn std::error::Error>> {\n     println!(\"Hello!\");\n+    Ok(())\n }",
  "changes_summary": "Added error handling return type and Ok(()) return",
  "message": "File edit completed successfully. Added error handling return type and Ok(()) return. Ready to apply changes or continue with next operation."
}
```

### 4. `directory.create` - Create Directories

Creates directories with optional recursive creation.

**Parameters:**
- `path` (string): Path to the directory to create
- `recursive` (boolean): Whether to create parent directories if they don't exist

**Example:**
```json
{
  "name": "directory.create",
  "arguments": {
    "path": "src/modules/auth",
    "recursive": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "path": "/workspace/src/modules/auth",
  "message": "Successfully created directory and any missing parent directories. Ready for next operation."
}
```

### 5. `directory.list` - List Directory Contents

Lists files and directories with detailed metadata.

**Parameters:**
- `path` (string): Path to the directory to list

**Example:**
```json
{
  "name": "directory.list",
  "arguments": {
    "path": "src"
  }
}
```

**Response:**
```json
{
  "entries": [
    {
      "name": "main.rs",
      "path": "src/main.rs",
      "type": "file",
      "size": 1024,
      "modified": "2024-01-15T10:30:00Z"
    },
    {
      "name": "lib.rs",
      "path": "src/lib.rs",
      "type": "file",
      "size": 2048,
      "modified": "2024-01-15T09:15:00Z"
    },
    {
      "name": "modules",
      "path": "src/modules",
      "type": "directory",
      "size": 0,
      "modified": "2024-01-15T08:45:00Z"
    }
  ],
  "total_files": 2,
  "total_directories": 1,
  "message": "Successfully listed directory contents: 2 files and 1 directories found. Information available for further operations."
}
```

## Key Features

### Continuation Signals
All tool responses include encouraging messages that signal to AI agents that they should continue with multi-step tasks:
- "Ready for next operation."
- "Content available for further processing."
- "Information available for further operations."

### Structured Responses
Unlike shell commands, these tools provide structured JSON responses that are easy for AI agents to parse and understand.

### Error Handling
Tools provide clear error messages and maintain consistent response formats even when operations fail.

### Security
All operations respect the configured sandbox policy and approval settings.

## Benefits Over Shell Commands

1. **Better UX**: Clear approval prompts instead of complex shell syntax
2. **Structured Data**: JSON responses instead of parsing text output
3. **Continuation Hints**: Built-in signals for multi-step workflows
4. **Error Clarity**: Structured error responses with actionable information
5. **Type Safety**: Well-defined request/response schemas