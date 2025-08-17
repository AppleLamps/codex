use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};

/// Represents a user-defined plugin tool
#[derive(Debug, Clone, PartialEq)]
pub struct Plugin {
    /// The name of the plugin tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// Path to the executable script or binary
    pub executable_path: PathBuf,
    /// JSON schema for the tool's parameters
    pub parameters: PluginParameters,
    /// The directory containing the plugin
    pub plugin_dir: PathBuf,
}

/// Plugin manifest structure that matches the TOML format
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PluginManifest {
    /// The name of the plugin tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// The script or binary to execute (relative to plugin directory)
    pub executable: String,
    /// JSON schema parameters for the tool
    pub parameters: PluginParameters,
}

/// Plugin parameters following JSON Schema structure
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PluginParameters {
    /// Type should be "object" for tool parameters
    #[serde(rename = "type")]
    pub param_type: String,
    /// Required parameter names
    #[serde(default)]
    pub required: Vec<String>,
    /// Parameter property definitions
    #[serde(default)]
    pub properties: HashMap<String, PluginProperty>,
}

/// Individual parameter property definition
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PluginProperty {
    /// Parameter type (string, number, boolean, etc.)
    #[serde(rename = "type")]
    pub prop_type: String,
    /// Description of the parameter
    #[serde(default)]
    pub description: Option<String>,
    /// Default value for the parameter
    #[serde(default)]
    pub default: Option<serde_json::Value>,
}

impl Default for PluginParameters {
    fn default() -> Self {
        Self {
            param_type: "object".to_string(),
            required: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

/// Discovers and loads plugins from a directory
pub fn discover_plugins(plugin_dir: &Path) -> Result<Vec<Plugin>> {
    if !plugin_dir.exists() {
        return Ok(Vec::new());
    }

    if !plugin_dir.is_dir() {
        return Err(anyhow::anyhow!("Plugin path is not a directory: {}", plugin_dir.display()));
    }

    let mut plugins = Vec::new();

    // Iterate through subdirectories in the plugin directory
    for entry in fs::read_dir(plugin_dir)
        .with_context(|| format!("Failed to read plugin directory: {}", plugin_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(plugin) = load_plugin_from_directory(&path) {
                plugins.push(plugin);
            } else {
                tracing::warn!("Failed to load plugin from directory: {}", path.display());
            }
        }
    }

    Ok(plugins)
}

/// Loads a single plugin from a directory containing plugin.toml and executable
fn load_plugin_from_directory(plugin_dir: &Path) -> Result<Plugin> {
    let manifest_path = plugin_dir.join("plugin.toml");
    
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!("Plugin manifest not found: {}", manifest_path.display()));
    }

    // Read and parse the plugin manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read plugin manifest: {}", manifest_path.display()))?;
    
    let manifest: PluginManifest = toml::from_str(&manifest_content)
        .with_context(|| format!("Failed to parse plugin manifest: {}", manifest_path.display()))?;

    // Resolve the executable path
    let executable_path = plugin_dir.join(&manifest.executable);
    
    if !executable_path.exists() {
        return Err(anyhow::anyhow!(
            "Plugin executable not found: {} (specified in {})",
            executable_path.display(),
            manifest_path.display()
        ));
    }

    Ok(Plugin {
        name: manifest.name,
        description: manifest.description,
        executable_path,
        parameters: manifest.parameters,
        plugin_dir: plugin_dir.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_plugin_manifest_parsing() {
        let toml_content = r#"
name = "my_custom_tool"
description = "A description of what this tool does."
executable = "run.py"

[parameters]
type = "object"
required = ["param1"]

[parameters.properties.param1]
type = "string"
description = "A description of the first parameter."

[parameters.properties.param2]
type = "number"
description = "An optional numeric parameter."
"#;

        let manifest: PluginManifest = toml::from_str(toml_content).unwrap();
        
        assert_eq!(manifest.name, "my_custom_tool");
        assert_eq!(manifest.description, "A description of what this tool does.");
        assert_eq!(manifest.executable, "run.py");
        assert_eq!(manifest.parameters.param_type, "object");
        assert_eq!(manifest.parameters.required, vec!["param1"]);
        assert_eq!(manifest.parameters.properties.len(), 2);
        
        let param1 = &manifest.parameters.properties["param1"];
        assert_eq!(param1.prop_type, "string");
        assert_eq!(param1.description, Some("A description of the first parameter.".to_string()));
        
        let param2 = &manifest.parameters.properties["param2"];
        assert_eq!(param2.prop_type, "number");
        assert_eq!(param2.description, Some("An optional numeric parameter.".to_string()));
    }

    #[test]
    fn test_discover_plugins_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let plugins = discover_plugins(temp_dir.path()).unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_discover_plugins_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");
        let plugins = discover_plugins(&nonexistent).unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_load_plugin_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let plugin_dir = temp_dir.path().join("test_plugin");
        fs::create_dir(&plugin_dir).unwrap();

        // Create plugin.toml
        let manifest_content = r#"
name = "test_tool"
description = "A test tool"
executable = "run.sh"

[parameters]
type = "object"
required = ["input"]

[parameters.properties.input]
type = "string"
description = "Input parameter"
"#;
        fs::write(plugin_dir.join("plugin.toml"), manifest_content).unwrap();

        // Create executable
        fs::write(plugin_dir.join("run.sh"), "#!/bin/bash\necho 'test'").unwrap();

        let plugin = load_plugin_from_directory(&plugin_dir).unwrap();
        
        assert_eq!(plugin.name, "test_tool");
        assert_eq!(plugin.description, "A test tool");
        assert_eq!(plugin.executable_path, plugin_dir.join("run.sh"));
        assert_eq!(plugin.parameters.required, vec!["input"]);
    }
}
