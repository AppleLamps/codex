//! LSP tools for AI agent code intelligence capabilities.
//!
//! This module defines the tool schemas that expose LSP functionality to the AI agent,
//! enabling advanced code intelligence features like completion, diagnostics, and navigation.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::openai_tools::{JsonSchema, OpenAiTool, ResponsesApiTool};

/// Arguments for the code.complete tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompleteArgs {
    /// Path to the file for completion
    pub file_path: String,
    /// Line number (0-based)
    pub line: u32,
    /// Character position in the line (0-based)
    pub character: u32,
    /// Optional context around the completion point
    pub context: Option<String>,
}

/// Arguments for the code.diagnostics tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDiagnosticsArgs {
    /// Path to the file to analyze
    pub file_path: String,
    /// Optional: specific line range to analyze [start_line, end_line]
    pub line_range: Option<[u32; 2]>,
}

/// Arguments for the code.definition tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDefinitionArgs {
    /// Path to the file
    pub file_path: String,
    /// Line number (0-based)
    pub line: u32,
    /// Character position in the line (0-based)
    pub character: u32,
    /// Symbol name for context
    pub symbol: Option<String>,
}

/// Arguments for the code.references tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReferencesArgs {
    /// Path to the file
    pub file_path: String,
    /// Line number (0-based)
    pub line: u32,
    /// Character position in the line (0-based)
    pub character: u32,
    /// Include declaration in results
    pub include_declaration: Option<bool>,
}

/// Arguments for the code.symbols tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSymbolsArgs {
    /// Path to the file or directory
    pub path: String,
    /// Type of symbols to find: "file" for document symbols, "workspace" for workspace symbols
    pub scope: String,
    /// Optional query to filter symbols
    pub query: Option<String>,
}

/// Arguments for the code.hover tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeHoverArgs {
    /// Path to the file
    pub file_path: String,
    /// Line number (0-based)
    pub line: u32,
    /// Character position in the line (0-based)
    pub character: u32,
}

/// Response for code completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeCompleteResponse {
    /// List of completion items
    pub completions: Vec<CompletionItem>,
    /// Whether the completion list is incomplete
    pub is_incomplete: bool,
}

/// Individual completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// The label of this completion item
    pub label: String,
    /// The kind of this completion item (1=Text, 2=Method, 3=Function, etc.)
    pub kind: Option<u32>,
    /// A human-readable string with additional information
    pub detail: Option<String>,
    /// Documentation for this completion item
    pub documentation: Option<String>,
    /// Text to insert when this completion is selected
    pub insert_text: Option<String>,
}

/// Response for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDiagnosticsResponse {
    /// List of diagnostic issues
    pub diagnostics: Vec<DiagnosticItem>,
    /// Summary of issues by severity
    pub summary: DiagnosticSummary,
}

/// Individual diagnostic item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    /// Line range of the diagnostic
    pub range: LineRange,
    /// Severity: 1=Error, 2=Warning, 3=Information, 4=Hint
    pub severity: u32,
    /// Diagnostic message
    pub message: String,
    /// Source of the diagnostic (e.g., "rust-analyzer", "eslint")
    pub source: Option<String>,
    /// Error code if available
    pub code: Option<String>,
}

/// Line range for diagnostics and other features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineRange {
    /// Start line (0-based)
    pub start_line: u32,
    /// Start character (0-based)
    pub start_character: u32,
    /// End line (0-based)
    pub end_line: u32,
    /// End character (0-based)
    pub end_character: u32,
}

/// Summary of diagnostic issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    /// Number of errors
    pub errors: u32,
    /// Number of warnings
    pub warnings: u32,
    /// Number of information messages
    pub info: u32,
    /// Number of hints
    pub hints: u32,
}

/// Response for go-to-definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDefinitionResponse {
    /// List of definition locations
    pub definitions: Vec<LocationItem>,
}

/// Location item for definitions and references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationItem {
    /// File path
    pub file_path: String,
    /// Line range
    pub range: LineRange,
    /// Preview of the code at this location
    pub preview: Option<String>,
}

/// Response for find references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReferencesResponse {
    /// List of reference locations
    pub references: Vec<LocationItem>,
    /// Total count of references
    pub total_count: u32,
}

/// Response for symbols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSymbolsResponse {
    /// List of symbols
    pub symbols: Vec<SymbolItem>,
}

/// Symbol item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolItem {
    /// Symbol name
    pub name: String,
    /// Symbol kind (1=File, 2=Module, 3=Namespace, 4=Package, 5=Class, 6=Method, etc.)
    pub kind: u32,
    /// File path containing the symbol
    pub file_path: String,
    /// Location range
    pub range: LineRange,
    /// Container name (e.g., class name for a method)
    pub container: Option<String>,
}

/// Response for hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeHoverResponse {
    /// Hover content
    pub content: String,
    /// Range that the hover applies to
    pub range: Option<LineRange>,
}

/// Creates the code.complete tool definition
pub(crate) fn create_code_complete_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "file_path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file for code completion".to_string()),
        },
    );
    
    properties.insert(
        "line".to_string(),
        JsonSchema::Number {
            description: Some("Line number (0-based) for completion".to_string()),
        },
    );
    
    properties.insert(
        "character".to_string(),
        JsonSchema::Number {
            description: Some("Character position (0-based) in the line".to_string()),
        },
    );
    
    properties.insert(
        "context".to_string(),
        JsonSchema::String {
            description: Some("Optional context around the completion point".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "code.complete".to_string(),
        description: "Get code completion suggestions at a specific position in a file. Provides intelligent autocomplete based on context, available symbols, and language semantics.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["file_path".to_string(), "line".to_string(), "character".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the code.diagnostics tool definition
pub(crate) fn create_code_diagnostics_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "file_path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file to analyze for issues".to_string()),
        },
    );
    
    properties.insert(
        "line_range".to_string(),
        JsonSchema::Array {
            items: Box::new(JsonSchema::Number { description: None }),
            description: Some("Optional line range [start, end] to analyze".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "code.diagnostics".to_string(),
        description: "Analyze code for errors, warnings, and other issues. Returns detailed diagnostic information including syntax errors, type errors, linting issues, and suggestions for improvement.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["file_path".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the code.definition tool definition
pub(crate) fn create_code_definition_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();
    
    properties.insert(
        "file_path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file containing the symbol".to_string()),
        },
    );
    
    properties.insert(
        "line".to_string(),
        JsonSchema::Number {
            description: Some("Line number (0-based) of the symbol".to_string()),
        },
    );
    
    properties.insert(
        "character".to_string(),
        JsonSchema::Number {
            description: Some("Character position (0-based) of the symbol".to_string()),
        },
    );
    
    properties.insert(
        "symbol".to_string(),
        JsonSchema::String {
            description: Some("Optional symbol name for context".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "code.definition".to_string(),
        description: "Go to the definition of a symbol at a specific position. Finds where functions, variables, types, and other symbols are defined.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["file_path".to_string(), "line".to_string(), "character".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the code.references tool definition
pub(crate) fn create_code_references_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();

    properties.insert(
        "file_path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file containing the symbol".to_string()),
        },
    );

    properties.insert(
        "line".to_string(),
        JsonSchema::Number {
            description: Some("Line number (0-based) of the symbol".to_string()),
        },
    );

    properties.insert(
        "character".to_string(),
        JsonSchema::Number {
            description: Some("Character position (0-based) of the symbol".to_string()),
        },
    );

    properties.insert(
        "include_declaration".to_string(),
        JsonSchema::Boolean {
            description: Some("Whether to include the declaration in results".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "code.references".to_string(),
        description: "Find all references to a symbol at a specific position. Shows where functions, variables, types, and other symbols are used throughout the codebase.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["file_path".to_string(), "line".to_string(), "character".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the code.symbols tool definition
pub(crate) fn create_code_symbols_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();

    properties.insert(
        "path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file or directory to search for symbols".to_string()),
        },
    );

    properties.insert(
        "scope".to_string(),
        JsonSchema::String {
            description: Some("Scope of search: 'file' for document symbols, 'workspace' for workspace symbols".to_string()),
        },
    );

    properties.insert(
        "query".to_string(),
        JsonSchema::String {
            description: Some("Optional query to filter symbols by name".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "code.symbols".to_string(),
        description: "Find symbols (functions, classes, variables, etc.) in a file or workspace. Useful for code navigation and understanding code structure.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["path".to_string(), "scope".to_string()]),
            additional_properties: Some(false),
        },
    })
}

/// Creates the code.hover tool definition
pub(crate) fn create_code_hover_tool() -> OpenAiTool {
    let mut properties = BTreeMap::new();

    properties.insert(
        "file_path".to_string(),
        JsonSchema::String {
            description: Some("Path to the file containing the symbol".to_string()),
        },
    );

    properties.insert(
        "line".to_string(),
        JsonSchema::Number {
            description: Some("Line number (0-based) of the symbol".to_string()),
        },
    );

    properties.insert(
        "character".to_string(),
        JsonSchema::Number {
            description: Some("Character position (0-based) of the symbol".to_string()),
        },
    );

    OpenAiTool::Function(ResponsesApiTool {
        name: "code.hover".to_string(),
        description: "Get hover information for a symbol at a specific position. Provides documentation, type information, and other details about the symbol.".to_string(),
        strict: false,
        parameters: JsonSchema::Object {
            properties,
            required: Some(vec!["file_path".to_string(), "line".to_string(), "character".to_string()]),
            additional_properties: Some(false),
        },
    })
}
