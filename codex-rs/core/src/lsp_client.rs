//! Language Server Protocol (LSP) client implementation for code intelligence.
//!
//! This module provides LSP client functionality to enable advanced code intelligence
//! features like completion, diagnostics, go-to-definition, and more.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// LSP client for communicating with language servers
#[derive(Clone)]
pub struct LspClient {
    inner: Arc<LspClientInner>,
}

struct LspClientInner {
    sender: mpsc::UnboundedSender<LspRequest>,
    language: String,
    root_uri: String,
}

/// LSP request with response channel
struct LspRequest {
    method: String,
    params: Value,
    response_tx: oneshot::Sender<Result<Value>>,
}

/// LSP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    /// Language identifier (e.g., "rust", "typescript", "python")
    pub language: String,
    /// Command to start the language server
    pub command: String,
    /// Arguments for the language server command
    pub args: Vec<String>,
    /// Environment variables for the language server
    pub env: HashMap<String, String>,
    /// File extensions this server handles
    pub file_extensions: Vec<String>,
}

/// LSP position in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// LSP range in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// LSP diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<u32>, // 1=Error, 2=Warning, 3=Information, 4=Hint
    pub code: Option<Value>,
    pub source: Option<String>,
    pub message: String,
}

/// LSP completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<u32>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
}

/// LSP location for go-to-definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// Default LSP server configurations for common languages
pub fn default_lsp_configs() -> Vec<LspServerConfig> {
    vec![
        // Rust
        LspServerConfig {
            language: "rust".to_string(),
            command: "rust-analyzer".to_string(),
            args: vec![],
            env: HashMap::new(),
            file_extensions: vec!["rs".to_string()],
        },
        // TypeScript/JavaScript
        LspServerConfig {
            language: "typescript".to_string(),
            command: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
            env: HashMap::new(),
            file_extensions: vec!["ts".to_string(), "tsx".to_string(), "js".to_string(), "jsx".to_string()],
        },
        // Python
        LspServerConfig {
            language: "python".to_string(),
            command: "pylsp".to_string(),
            args: vec![],
            env: HashMap::new(),
            file_extensions: vec!["py".to_string(), "pyi".to_string()],
        },
        // Go
        LspServerConfig {
            language: "go".to_string(),
            command: "gopls".to_string(),
            args: vec![],
            env: HashMap::new(),
            file_extensions: vec!["go".to_string()],
        },
        // Java
        LspServerConfig {
            language: "java".to_string(),
            command: "jdtls".to_string(),
            args: vec![],
            env: HashMap::new(),
            file_extensions: vec!["java".to_string()],
        },
    ]
}

impl LspClient {
    /// Create a new LSP client for the specified language and workspace
    pub async fn new(
        config: &LspServerConfig,
        workspace_root: &Path,
    ) -> Result<Self> {
        let root_uri = format!("file://{}", workspace_root.to_string_lossy());
        
        info!("Starting LSP server for {}: {} {:?}", 
              config.language, config.command, config.args);

        // Start the language server process
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .envs(&config.env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to start LSP server: {}", config.command))?;

        let stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        // Create communication channels
        let (request_tx, request_rx) = mpsc::unbounded_channel();

        // Spawn the LSP communication task
        let language = config.language.clone();
        let root_uri_clone = root_uri.clone();
        tokio::spawn(lsp_communication_task(
            child,
            stdin,
            stdout,
            request_rx,
            language.clone(),
            root_uri_clone,
        ));

        let inner = Arc::new(LspClientInner {
            sender: request_tx,
            language,
            root_uri,
        });

        Ok(LspClient { inner })
    }

    /// Get code completion at the specified position
    pub async fn get_completion(
        &self,
        file_path: &Path,
        position: Position,
    ) -> Result<Vec<CompletionItem>> {
        let uri = format!("file://{}", file_path.to_string_lossy());
        
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": position
        });

        let response = self.send_request("textDocument/completion", params).await?;
        
        // Parse completion response
        if let Some(items) = response.get("items") {
            let completions: Vec<CompletionItem> = serde_json::from_value(items.clone())
                .unwrap_or_default();
            Ok(completions)
        } else if let Ok(completions) = serde_json::from_value::<Vec<CompletionItem>>(response) {
            Ok(completions)
        } else {
            Ok(vec![])
        }
    }

    /// Get diagnostics for a file
    pub async fn get_diagnostics(&self, file_path: &Path) -> Result<Vec<Diagnostic>> {
        let uri = format!("file://{}", file_path.to_string_lossy());
        
        // For diagnostics, we typically need to open the document first
        // This is a simplified implementation
        let params = json!({
            "textDocument": {
                "uri": uri
            }
        });

        // Note: Diagnostics are usually pushed by the server, not pulled
        // This is a simplified approach for demonstration
        match self.send_request("textDocument/publishDiagnostics", params).await {
            Ok(response) => {
                if let Some(diagnostics) = response.get("diagnostics") {
                    let diags: Vec<Diagnostic> = serde_json::from_value(diagnostics.clone())
                        .unwrap_or_default();
                    Ok(diags)
                } else {
                    Ok(vec![])
                }
            }
            Err(_) => Ok(vec![]) // Diagnostics might not be available immediately
        }
    }

    /// Go to definition at the specified position
    pub async fn go_to_definition(
        &self,
        file_path: &Path,
        position: Position,
    ) -> Result<Vec<Location>> {
        let uri = format!("file://{}", file_path.to_string_lossy());
        
        let params = json!({
            "textDocument": {
                "uri": uri
            },
            "position": position
        });

        let response = self.send_request("textDocument/definition", params).await?;
        
        // Parse definition response (can be Location, Location[], or LocationLink[])
        if let Ok(location) = serde_json::from_value::<Location>(response.clone()) {
            Ok(vec![location])
        } else if let Ok(locations) = serde_json::from_value::<Vec<Location>>(response) {
            Ok(locations)
        } else {
            Ok(vec![])
        }
    }

    /// Send a request to the LSP server
    async fn send_request(&self, method: &str, params: Value) -> Result<Value> {
        let (response_tx, response_rx) = oneshot::channel();
        
        let request = LspRequest {
            method: method.to_string(),
            params,
            response_tx,
        };

        self.inner.sender.send(request)
            .map_err(|_| anyhow::anyhow!("LSP client is closed"))?;

        // Wait for response with timeout
        timeout(Duration::from_secs(10), response_rx)
            .await
            .map_err(|_| anyhow::anyhow!("LSP request timed out"))?
            .map_err(|_| anyhow::anyhow!("LSP request was cancelled"))?
    }

    /// Get the language this client handles
    pub fn language(&self) -> &str {
        &self.inner.language
    }

    /// Get the workspace root URI
    pub fn root_uri(&self) -> &str {
        &self.inner.root_uri
    }
}

/// LSP communication task that handles the JSON-RPC protocol
async fn lsp_communication_task(
    mut child: Child,
    mut stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    mut request_rx: mpsc::UnboundedReceiver<LspRequest>,
    language: String,
    root_uri: String,
) {
    let mut reader = BufReader::new(stdout);
    let mut request_id = 0u64;
    let pending_requests = Arc::new(Mutex::new(HashMap::<u64, oneshot::Sender<Result<Value>>>::new()));

    // Initialize the LSP server
    if let Err(e) = initialize_lsp_server(&mut stdin, &mut reader, &root_uri, &mut request_id).await {
        error!("Failed to initialize LSP server for {}: {}", language, e);
        return;
    }

    info!("LSP server for {} initialized successfully", language);

    // Spawn task to handle incoming responses
    let pending_requests_clone = pending_requests.clone();
    tokio::spawn(async move {
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Err(e) = handle_lsp_message(&line, &pending_requests_clone).await {
                        debug!("Error handling LSP message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error reading from LSP server: {}", e);
                    break;
                }
            }
        }
    });

    // Handle outgoing requests
    while let Some(request) = request_rx.recv().await {
        request_id += 1;

        // Store the response channel
        {
            let mut pending = pending_requests.lock().await;
            pending.insert(request_id, request.response_tx);
        }

        // Send the request
        let message = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": request.method,
            "params": request.params
        });

        if let Err(e) = send_lsp_message(&mut stdin, &message).await {
            error!("Failed to send LSP request: {}", e);

            // Remove and notify the pending request
            let mut pending = pending_requests.lock().await;
            if let Some(tx) = pending.remove(&request_id) {
                let _ = tx.send(Err(anyhow::anyhow!("Failed to send request: {}", e)));
            }
        }
    }

    // Clean up
    let _ = child.kill().await;
}

/// Initialize the LSP server with the initialize request
async fn initialize_lsp_server(
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut BufReader<tokio::process::ChildStdout>,
    root_uri: &str,
    request_id: &mut u64,
) -> Result<()> {
    *request_id += 1;

    let initialize_params = json!({
        "processId": std::process::id(),
        "rootUri": root_uri,
        "capabilities": {
            "textDocument": {
                "completion": {
                    "completionItem": {
                        "snippetSupport": true,
                        "documentationFormat": ["markdown", "plaintext"]
                    }
                },
                "definition": {
                    "linkSupport": true
                },
                "publishDiagnostics": {}
            },
            "workspace": {
                "workspaceFolders": true
            }
        },
        "workspaceFolders": [{
            "uri": root_uri,
            "name": "workspace"
        }]
    });

    let message = json!({
        "jsonrpc": "2.0",
        "id": *request_id,
        "method": "initialize",
        "params": initialize_params
    });

    send_lsp_message(stdin, &message).await?;

    // Wait for initialize response
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    // Send initialized notification
    let initialized = json!({
        "jsonrpc": "2.0",
        "method": "initialized",
        "params": {}
    });

    send_lsp_message(stdin, &initialized).await?;

    Ok(())
}

/// Send an LSP message using the JSON-RPC protocol
async fn send_lsp_message(
    stdin: &mut tokio::process::ChildStdin,
    message: &Value,
) -> Result<()> {
    let content = serde_json::to_string(message)?;
    let header = format!("Content-Length: {}\r\n\r\n", content.len());

    stdin.write_all(header.as_bytes()).await?;
    stdin.write_all(content.as_bytes()).await?;
    stdin.flush().await?;

    Ok(())
}

/// Handle incoming LSP messages and route responses to pending requests
async fn handle_lsp_message(
    line: &str,
    pending_requests: &Arc<Mutex<HashMap<u64, oneshot::Sender<Result<Value>>>>>,
) -> Result<()> {
    // Skip empty lines and headers
    if line.trim().is_empty() || line.starts_with("Content-Length:") {
        return Ok(());
    }

    let message: Value = serde_json::from_str(line.trim())?;

    // Handle responses to our requests
    if let Some(id) = message.get("id") {
        if let Ok(request_id) = serde_json::from_value::<u64>(id.clone()) {
            let mut pending = pending_requests.lock().await;
            if let Some(tx) = pending.remove(&request_id) {
                if let Some(error) = message.get("error") {
                    let _ = tx.send(Err(anyhow::anyhow!("LSP error: {}", error)));
                } else if let Some(result) = message.get("result") {
                    let _ = tx.send(Ok(result.clone()));
                } else {
                    let _ = tx.send(Err(anyhow::anyhow!("Invalid LSP response")));
                }
            }
        }
    }

    // Handle notifications (diagnostics, etc.)
    if let Some(method) = message.get("method") {
        if method == "textDocument/publishDiagnostics" {
            debug!("Received diagnostics: {}", message);
            // In a full implementation, we'd store these diagnostics
        }
    }

    Ok(())
}
