//! LSP tool handlers for processing AI agent requests.
//!
//! This module implements the actual logic for handling LSP tool calls,
//! managing LSP clients, and formatting responses for the AI agent.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::lsp_client::{LspClient, LspServerConfig, Position, default_lsp_configs};
use crate::lsp_tools::*;
use crate::models::FunctionCallOutputPayload;

/// LSP manager that handles multiple language servers
pub struct LspManager {
    clients: Arc<RwLock<HashMap<String, LspClient>>>,
    configs: Vec<LspServerConfig>,
    workspace_root: PathBuf,
}

impl LspManager {
    /// Create a new LSP manager
    pub fn new(workspace_root: PathBuf) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            configs: default_lsp_configs(),
            workspace_root,
        }
    }

    /// Get the workspace root
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Get or create an LSP client for the given file
    async fn get_client_for_file(&self, file_path: &Path) -> Result<LspClient> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        // Find the appropriate language server config
        let config = self.configs
            .iter()
            .find(|cfg| cfg.file_extensions.contains(&extension.to_string()))
            .ok_or_else(|| anyhow::anyhow!("No LSP server configured for file extension: {}", extension))?;

        let language = &config.language;

        // Check if we already have a client for this language
        {
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(language) {
                return Ok(client.clone());
            }
        }

        // Create a new client
        info!("Starting LSP client for language: {}", language);
        let client = LspClient::new(config, &self.workspace_root).await
            .with_context(|| format!("Failed to start LSP client for {}", language))?;

        // Store the client
        {
            let mut clients = self.clients.write().await;
            clients.insert(language.clone(), client.clone());
        }

        Ok(client)
    }

    /// Get language for a file based on extension
    fn get_language_for_file(&self, file_path: &Path) -> Option<String> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())?;

        self.configs
            .iter()
            .find(|cfg| cfg.file_extensions.contains(&extension.to_string()))
            .map(|cfg| cfg.language.clone())
    }
}

/// Handle code completion requests
pub async fn handle_code_complete(
    args: CodeCompleteArgs,
    lsp_manager: &LspManager,
) -> Result<FunctionCallOutputPayload> {
    let file_path = Path::new(&args.file_path);
    
    match lsp_manager.get_client_for_file(file_path).await {
        Ok(client) => {
            let position = Position {
                line: args.line,
                character: args.character,
            };

            match client.get_completion(file_path, position).await {
                Ok(completions) => {
                    let response = CodeCompleteResponse {
                        completions: completions.into_iter().map(|item| CompletionItem {
                            label: item.label,
                            kind: item.kind,
                            detail: item.detail,
                            documentation: item.documentation,
                            insert_text: item.insert_text,
                        }).collect(),
                        is_incomplete: false,
                    };

                    Ok(FunctionCallOutputPayload {
                        content: serde_json::to_string_pretty(&response)?,
                        success: Some(true),
                    })
                }
                Err(e) => {
                    warn!("LSP completion failed: {}", e);
                    Ok(FunctionCallOutputPayload {
                        content: format!("Code completion failed: {}", e),
                        success: Some(false),
                    })
                }
            }
        }
        Err(e) => {
            let language = lsp_manager.get_language_for_file(file_path)
                .unwrap_or_else(|| "unknown".to_string());
            
            Ok(FunctionCallOutputPayload {
                content: format!("No LSP server available for {} files. Error: {}", language, e),
                success: Some(false),
            })
        }
    }
}

/// Handle code diagnostics requests
pub async fn handle_code_diagnostics(
    args: CodeDiagnosticsArgs,
    lsp_manager: &LspManager,
) -> Result<FunctionCallOutputPayload> {
    let file_path = Path::new(&args.file_path);
    
    match lsp_manager.get_client_for_file(file_path).await {
        Ok(client) => {
            match client.get_diagnostics(file_path).await {
                Ok(diagnostics) => {
                    let mut errors = 0;
                    let mut warnings = 0;
                    let mut info = 0;
                    let mut hints = 0;

                    let diagnostic_items: Vec<DiagnosticItem> = diagnostics.into_iter().map(|diag| {
                        let severity = diag.severity.unwrap_or(1);
                        match severity {
                            1 => errors += 1,
                            2 => warnings += 1,
                            3 => info += 1,
                            4 => hints += 1,
                            _ => {}
                        }

                        DiagnosticItem {
                            range: LineRange {
                                start_line: diag.range.start.line,
                                start_character: diag.range.start.character,
                                end_line: diag.range.end.line,
                                end_character: diag.range.end.character,
                            },
                            severity,
                            message: diag.message,
                            source: diag.source,
                            code: diag.code.and_then(|c| c.as_str().map(|s| s.to_string())),
                        }
                    }).collect();

                    let response = CodeDiagnosticsResponse {
                        diagnostics: diagnostic_items,
                        summary: DiagnosticSummary {
                            errors,
                            warnings,
                            info,
                            hints,
                        },
                    };

                    Ok(FunctionCallOutputPayload {
                        content: serde_json::to_string_pretty(&response)?,
                        success: Some(true),
                    })
                }
                Err(e) => {
                    warn!("LSP diagnostics failed: {}", e);
                    Ok(FunctionCallOutputPayload {
                        content: format!("Code diagnostics failed: {}", e),
                        success: Some(false),
                    })
                }
            }
        }
        Err(e) => {
            let language = lsp_manager.get_language_for_file(file_path)
                .unwrap_or_else(|| "unknown".to_string());
            
            Ok(FunctionCallOutputPayload {
                content: format!("No LSP server available for {} files. Error: {}", language, e),
                success: Some(false),
            })
        }
    }
}

/// Handle go-to-definition requests
pub async fn handle_code_definition(
    args: CodeDefinitionArgs,
    lsp_manager: &LspManager,
) -> Result<FunctionCallOutputPayload> {
    let file_path = Path::new(&args.file_path);
    
    match lsp_manager.get_client_for_file(file_path).await {
        Ok(client) => {
            let position = Position {
                line: args.line,
                character: args.character,
            };

            match client.go_to_definition(file_path, position).await {
                Ok(locations) => {
                    let definitions: Vec<LocationItem> = locations.into_iter().map(|loc| {
                        // Extract file path from URI
                        let file_path = loc.uri.strip_prefix("file://")
                            .unwrap_or(&loc.uri)
                            .to_string();

                        LocationItem {
                            file_path,
                            range: LineRange {
                                start_line: loc.range.start.line,
                                start_character: loc.range.start.character,
                                end_line: loc.range.end.line,
                                end_character: loc.range.end.character,
                            },
                            preview: None, // Could be enhanced to read file content
                        }
                    }).collect();

                    let response = CodeDefinitionResponse { definitions };

                    Ok(FunctionCallOutputPayload {
                        content: serde_json::to_string_pretty(&response)?,
                        success: Some(true),
                    })
                }
                Err(e) => {
                    warn!("LSP go-to-definition failed: {}", e);
                    Ok(FunctionCallOutputPayload {
                        content: format!("Go-to-definition failed: {}", e),
                        success: Some(false),
                    })
                }
            }
        }
        Err(e) => {
            let language = lsp_manager.get_language_for_file(file_path)
                .unwrap_or_else(|| "unknown".to_string());
            
            Ok(FunctionCallOutputPayload {
                content: format!("No LSP server available for {} files. Error: {}", language, e),
                success: Some(false),
            })
        }
    }
}

/// Handle find references requests
pub async fn handle_code_references(
    args: CodeReferencesArgs,
    lsp_manager: &LspManager,
) -> Result<FunctionCallOutputPayload> {
    // This is a simplified implementation - full LSP references support would require
    // additional LSP client methods
    let file_path = Path::new(&args.file_path);
    
    match lsp_manager.get_client_for_file(file_path).await {
        Ok(_client) => {
            // For now, return a placeholder response
            let response = CodeReferencesResponse {
                references: vec![],
                total_count: 0,
            };

            Ok(FunctionCallOutputPayload {
                content: format!("Find references not yet fully implemented. File: {}, Position: {}:{}", 
                    args.file_path, args.line, args.character),
                success: Some(false),
            })
        }
        Err(e) => {
            let language = lsp_manager.get_language_for_file(file_path)
                .unwrap_or_else(|| "unknown".to_string());
            
            Ok(FunctionCallOutputPayload {
                content: format!("No LSP server available for {} files. Error: {}", language, e),
                success: Some(false),
            })
        }
    }
}

/// Handle symbol search requests
pub async fn handle_code_symbols(
    args: CodeSymbolsArgs,
    lsp_manager: &LspManager,
) -> Result<FunctionCallOutputPayload> {
    // This is a simplified implementation - full LSP symbols support would require
    // additional LSP client methods
    let path = Path::new(&args.path);
    
    if args.scope == "file" && path.is_file() {
        match lsp_manager.get_client_for_file(path).await {
            Ok(_client) => {
                // For now, return a placeholder response
                let response = CodeSymbolsResponse {
                    symbols: vec![],
                };

                Ok(FunctionCallOutputPayload {
                    content: format!("Symbol search not yet fully implemented. Path: {}, Scope: {}", 
                        args.path, args.scope),
                    success: Some(false),
                })
            }
            Err(e) => {
                let language = lsp_manager.get_language_for_file(path)
                    .unwrap_or_else(|| "unknown".to_string());
                
                Ok(FunctionCallOutputPayload {
                    content: format!("No LSP server available for {} files. Error: {}", language, e),
                    success: Some(false),
                })
            }
        }
    } else {
        Ok(FunctionCallOutputPayload {
            content: format!("Workspace symbol search not yet implemented. Path: {}", args.path),
            success: Some(false),
        })
    }
}

/// Handle hover information requests
pub async fn handle_code_hover(
    args: CodeHoverArgs,
    lsp_manager: &LspManager,
) -> Result<FunctionCallOutputPayload> {
    // This is a simplified implementation - full LSP hover support would require
    // additional LSP client methods
    let file_path = Path::new(&args.file_path);
    
    match lsp_manager.get_client_for_file(file_path).await {
        Ok(_client) => {
            // For now, return a placeholder response
            Ok(FunctionCallOutputPayload {
                content: format!("Hover information not yet fully implemented. File: {}, Position: {}:{}", 
                    args.file_path, args.line, args.character),
                success: Some(false),
            })
        }
        Err(e) => {
            let language = lsp_manager.get_language_for_file(file_path)
                .unwrap_or_else(|| "unknown".to_string());
            
            Ok(FunctionCallOutputPayload {
                content: format!("No LSP server available for {} files. Error: {}", language, e),
                success: Some(false),
            })
        }
    }
}
