use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::openai_tools::{JsonSchema, OpenAiTool, ResponsesApiTool};

/// Arguments for the file.create tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreateArgs {
    /// Path to the file to create (relative paths preferred)
    pub path: String,
    /// Optional content for the file (defaults to empty)
    #[serde(default)]
    pub content: Option<String>,
}

/// Arguments for the file.read tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadArgs {
    /// Path to the file to read
    pub path: String,
    /// Optional line range [start, end] (1-based, inclusive)
    pub lines: Option<[u32; 2]>,
}

/// Arguments for the file.edit tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditArgs {
    /// Path to the file to edit
    pub path: String,
    /// Human-readable instructions for the edit
    pub instructions: String,
    /// Optional specific line range to focus the edit on
    pub target_lines: Option<[u32; 2]>,
}

/// Arguments for the directory.create tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryCreateArgs {
    /// Path to the directory to create
    pub path: String,
    /// Whether to create parent directories if they don't exist
    #[serde(default)]
    pub recursive: bool,
}

/// Arguments for the directory.list tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListArgs {
    /// Path to the directory to list
    pub path: String,
    /// Whether to list contents recursively
    #[serde(default)]
    pub recursive: bool,
    /// Maximum depth for recursive listing (default: 2)
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,
}

fn default_max_depth() -> u32 {
    2
}

/// Response for file.read tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadResponse {
    pub content: String,
    pub total_lines: u32,
    pub encoding: String,
    pub size_bytes: u64,
    pub message: String,
}

/// Response for directory.list tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListResponse {
    pub entries: Vec<DirectoryEntry>,
    pub total_files: u32,
    pub total_directories: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub size_bytes: Option<u64>,
    pub modified: Option<String>, // ISO 8601 timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    File,
    Directory,
    Symlink,
}

/// Response for file.edit tool showing the proposed changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditResponse {
    pub original_content: String,
    pub new_content: String,
    pub diff: String,
    pub changes_summary: String,
    pub message: String,
}

/// Success response for file.create and directory.create
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResponse {
    pub success: bool,
    pub path: String,
    pub message: String,
}

/// Creates the file.create tool definition
pub fn create_file_create_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file to create (relative paths preferred)".to_string()),
        },
    );

    properties.insert(
        "content".to_string(),
        JsonSchema::String {
            description: Some("Content for the new file (optional, defaults to empty)".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "file.create".to_string(),
        description: "Create a new file with optional content. Creates parent directories if needed.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["path".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the file.read tool definition
pub fn create_file_read_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file to read".to_string()),
        },
    );

    properties.insert(
        "lines".to_string(),
        JsonSchema::Array {
            description: Some("Optional line range [start, end] (1-based, inclusive)".to_string()),
            items: Box::new(JsonSchema::Number {
                description: None,
            }),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "file.read".to_string(),
        description: "Read the contents of a file with optional line range specification.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["path".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the file.edit tool definition
pub fn create_file_edit_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file to edit".to_string()),
        },
    );

    properties.insert(
        "instructions".to_string(),
        JsonSchema::String {
            description: Some("Human-readable instructions describing the changes to make".to_string()),
        },
    );

    properties.insert(
        "target_lines".to_string(),
        JsonSchema::Array {
            description: Some("Optional line range [start, end] to focus the edit on".to_string()),
            items: Box::new(JsonSchema::Number {
                description: None,
            }),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "file.edit".to_string(),
        description: "Edit a file based on natural language instructions. Shows diff for approval.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["path".to_string(), "instructions".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the directory.create tool definition
pub fn create_directory_create_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "path".to_string(),
        JsonSchema::String {
            description: Some("Path to the directory to create".to_string()),
        },
    );
    
    properties.insert(
        "recursive".to_string(),
        JsonSchema::Boolean {
            description: Some("Whether to create parent directories if they don't exist".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "directory.create".to_string(),
        description: "Create a new directory with optional recursive parent creation.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["path".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the directory.list tool definition
pub fn create_directory_list_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "path".to_string(),
        JsonSchema::String {
            description: Some("Path to the directory to list".to_string()),
        },
    );

    properties.insert(
        "recursive".to_string(),
        JsonSchema::Boolean {
            description: Some("Whether to list contents recursively".to_string()),
        },
    );

    properties.insert(
        "max_depth".to_string(),
        JsonSchema::Number {
            description: Some("Maximum depth for recursive listing (default: 2)".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "directory.list".to_string(),
        description: "List directory contents with structured output and optional recursion.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["path".to_string()]),
            additional_properties: Some(false),
        },
    })
}
