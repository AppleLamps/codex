use std::sync::Arc;
use std::sync::Mutex;

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;

use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use ratatui::widgets::Wrap;

use crate::app::ChatWidgetArgs;
use crate::colors::LIGHT_BLUE;
use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepStateProvider;

use super::onboarding_screen::StepState;

#[derive(Clone, Debug)]
pub(crate) struct ModelOption {
    pub name: String,
    pub provider: String,
    pub provider_id: String,
    pub description: String,
    pub context_info: String,
    pub is_free: bool,
    pub requires_api_key: bool,
    pub env_key: Option<String>,
}

pub(crate) struct ModelSelectionWidget {
    pub chat_widget_args: Arc<Mutex<ChatWidgetArgs>>,
    pub available_models: Vec<ModelOption>,
    pub selected_index: Option<usize>,
    pub highlighted_index: usize,
    pub error: Option<String>,
}

impl ModelSelectionWidget {
    pub fn new(chat_widget_args: Arc<Mutex<ChatWidgetArgs>>) -> Self {
        let available_models = Self::get_available_models();
        
        Self {
            chat_widget_args,
            available_models,
            selected_index: None,
            highlighted_index: 0,
            error: None,
        }
    }

    fn get_available_models() -> Vec<ModelOption> {
        vec![
            ModelOption {
                name: "gpt-4o".to_string(),
                provider: "OpenAI".to_string(),
                provider_id: "openai".to_string(),
                description: "Fast, reliable, excellent for general coding tasks".to_string(),
                context_info: "128k context".to_string(),
                is_free: false,
                requires_api_key: true,
                env_key: Some("OPENAI_API_KEY".to_string()),
            },
            ModelOption {
                name: "o3".to_string(),
                provider: "OpenAI".to_string(),
                provider_id: "openai".to_string(),
                description: "Advanced reasoning, best for complex problems".to_string(),
                context_info: "200k context".to_string(),
                is_free: false,
                requires_api_key: true,
                env_key: Some("OPENAI_API_KEY".to_string()),
            },
            ModelOption {
                name: "openai/gpt-5".to_string(),
                provider: "OpenRouter".to_string(),
                provider_id: "openrouter".to_string(),
                description: "Cutting-edge model via OpenRouter".to_string(),
                context_info: "400k context, 180k output".to_string(),
                is_free: false,
                requires_api_key: true,
                env_key: Some("OPENROUTER_API_KEY".to_string()),
            },
            ModelOption {
                name: "anthropic/claude-sonnet-4".to_string(),
                provider: "OpenRouter".to_string(),
                provider_id: "openrouter".to_string(),
                description: "Excellent for code analysis and review".to_string(),
                context_info: "200k context, 64k output".to_string(),
                is_free: false,
                requires_api_key: true,
                env_key: Some("OPENROUTER_API_KEY".to_string()),
            },
            ModelOption {
                name: "qwen/qwen3-coder:free".to_string(),
                provider: "OpenRouter".to_string(),
                provider_id: "openrouter".to_string(),
                description: "Free coding model with large context".to_string(),
                context_info: "262k context, 262k output (FREE)".to_string(),
                is_free: true,
                requires_api_key: true,
                env_key: Some("OPENROUTER_API_KEY".to_string()),
            },
            ModelOption {
                name: "moonshotai/kimi-k2:free".to_string(),
                provider: "OpenRouter".to_string(),
                provider_id: "openrouter".to_string(),
                description: "Free general-purpose model".to_string(),
                context_info: "65.5k context, 65.5k output (FREE)".to_string(),
                is_free: true,
                requires_api_key: true,
                env_key: Some("OPENROUTER_API_KEY".to_string()),
            },
            ModelOption {
                name: "grok-4".to_string(),
                provider: "xAI".to_string(),
                provider_id: "xai".to_string(),
                description: "Alternative provider with unique capabilities".to_string(),
                context_info: "128k context".to_string(),
                is_free: false,
                requires_api_key: true,
                env_key: Some("XAI_API_KEY".to_string()),
            },
            ModelOption {
                name: "gpt-oss:20b".to_string(),
                provider: "Local OSS".to_string(),
                provider_id: "oss".to_string(),
                description: "Run models locally via Ollama (no API key needed)".to_string(),
                context_info: "Local execution".to_string(),
                is_free: true,
                requires_api_key: false,
                env_key: None,
            },
        ]
    }

    fn handle_selection(&mut self) {
        if let Some(model) = self.available_models.get(self.highlighted_index) {
            // Update the ChatWidgetArgs with the selected model
            if let Ok(mut args) = self.chat_widget_args.lock() {
                args.config.model = model.name.clone();
                args.config.model_provider_id = model.provider_id.clone();

                // Update the model provider info to match the selected provider
                if let Some(provider_info) = args.config.model_providers.get(&model.provider_id) {
                    args.config.model_provider = provider_info.clone();
                }

                // Update the model family
                if let Some(family) = codex_core::model_family::find_family_for_model(&model.name) {
                    args.config.model_family = family;
                }
            }

            self.selected_index = Some(self.highlighted_index);
        }
    }
}

impl WidgetRef for &ModelSelectionWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                Span::raw("> "),
                Span::styled(
                    "Select your preferred AI model:",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
        ];

        for (idx, model) in self.available_models.iter().enumerate() {
            let is_highlighted = idx == self.highlighted_index;
            let is_selected = self.selected_index == Some(idx);
            
            let bullet = if is_selected {
                "✓"
            } else if is_highlighted {
                "●"
            } else {
                " "
            };
            
            let number = format!("{}.", idx + 1);
            let bullet_style = if is_highlighted {
                Style::default().fg(LIGHT_BLUE).add_modifier(Modifier::BOLD)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };
            
            let name_style = if is_highlighted {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let provider_style = if is_highlighted {
                Style::default().fg(LIGHT_BLUE)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };

            // Model name line
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(number, bullet_style),
                Span::raw(" "),
                Span::styled(bullet, bullet_style),
                Span::raw(" "),
                Span::styled(&model.name, name_style),
                Span::raw(" ("),
                Span::styled(&model.provider, provider_style),
                Span::raw(")"),
            ]));
            
            // Description line
            let desc_style = if is_highlighted {
                Style::default()
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };
            
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(&model.description, desc_style),
            ]));
            
            // Context info line
            let context_style = if model.is_free {
                Style::default().fg(Color::Green)
            } else if is_highlighted {
                Style::default().fg(LIGHT_BLUE)
            } else {
                Style::default().add_modifier(Modifier::DIM)
            };
            
            lines.push(Line::from(vec![
                Span::raw("     "),
                Span::styled(&model.context_info, context_style),
                if model.requires_api_key {
                    if let Some(env_key) = &model.env_key {
                        Span::styled(
                            format!(" (requires {})", env_key),
                            Style::default().add_modifier(Modifier::DIM)
                        )
                    } else {
                        Span::raw("")
                    }
                } else {
                    Span::raw("")
                }
            ]));
            
            lines.push(Line::from(""));
        }

        lines.push(Line::from(""));
        lines.push(
            Line::from("  Use ↑↓ or j/k to navigate, Enter to select")
                .style(Style::default().add_modifier(Modifier::DIM)),
        );

        if let Some(err) = &self.error {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                err.as_str(),
                Style::default().fg(Color::Red),
            )));
        }

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl KeyboardHandler for ModelSelectionWidget {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.highlighted_index > 0 {
                    self.highlighted_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.highlighted_index < self.available_models.len() - 1 {
                    self.highlighted_index += 1;
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    let index = (digit as usize).saturating_sub(1);
                    if index < self.available_models.len() {
                        self.highlighted_index = index;
                        self.handle_selection();
                    }
                }
            }
            KeyCode::Enter => {
                self.handle_selection();
            }
            _ => {}
        }
    }
}

impl StepStateProvider for ModelSelectionWidget {
    fn get_step_state(&self) -> StepState {
        match self.selected_index {
            Some(_) => StepState::Complete,
            None => StepState::InProgress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_widget() -> ModelSelectionWidget {
        use crate::app::ChatWidgetArgs;
        use codex_core::config::{Config, ConfigOverrides, ConfigToml};
        use tempfile::TempDir;

        // Create a minimal config for testing
        let temp_dir = TempDir::new().unwrap();
        let config = Config::load_from_base_config_with_overrides(
            ConfigToml::default(),
            ConfigOverrides::default(),
            temp_dir.path().to_path_buf(),
        ).unwrap();

        // Create a test ChatWidgetArgs using the test constructor
        let chat_args = ChatWidgetArgs::new_for_test(config);

        ModelSelectionWidget::new(Arc::new(Mutex::new(chat_args)))
    }

    #[test]
    fn test_model_selection_widget_creation() {
        let widget = create_test_widget();
        assert_eq!(widget.highlighted_index, 0);
        assert_eq!(widget.selected_index, None);
        assert_eq!(widget.available_models.len(), 8); // We defined 8 models
        assert!(widget.error.is_none());
    }

    #[test]
    fn test_keyboard_navigation() {
        let mut widget = create_test_widget();

        // Test down navigation
        widget.handle_key_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        assert_eq!(widget.highlighted_index, 1);

        // Test up navigation
        widget.handle_key_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE));
        assert_eq!(widget.highlighted_index, 0);

        // Test j/k navigation
        widget.handle_key_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        assert_eq!(widget.highlighted_index, 1);

        widget.handle_key_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE));
        assert_eq!(widget.highlighted_index, 0);
    }

    #[test]
    fn test_model_selection() {
        let mut widget = create_test_widget();

        // Initially no selection
        assert_eq!(widget.get_step_state(), StepState::InProgress);

        // Select first model with Enter
        widget.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(widget.selected_index, Some(0));
        assert_eq!(widget.get_step_state(), StepState::Complete);

        // Verify the config was updated
        if let Ok(args) = widget.chat_widget_args.lock() {
            assert_eq!(args.config.model, "gpt-4o");
            assert_eq!(args.config.model_provider_id, "openai");
        }
    }

    #[test]
    fn test_number_key_selection() {
        let mut widget = create_test_widget();

        // Select third model with number key
        widget.handle_key_event(KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE));
        assert_eq!(widget.selected_index, Some(2));
        assert_eq!(widget.highlighted_index, 2);

        // Verify the config was updated to OpenRouter GPT-5
        if let Ok(args) = widget.chat_widget_args.lock() {
            assert_eq!(args.config.model, "openai/gpt-5");
            assert_eq!(args.config.model_provider_id, "openrouter");
        }
    }

    #[test]
    fn test_openrouter_model_provider_update() {
        let mut widget = create_test_widget();

        // Find the index of the moonshotai/kimi-k2:free model (OpenRouter)
        let kimi_index = widget.available_models.iter().position(|m| m.name == "moonshotai/kimi-k2:free").unwrap();

        // Select the kimi model
        widget.highlighted_index = kimi_index;
        widget.handle_selection();

        // Verify the config was updated correctly
        if let Ok(args) = widget.chat_widget_args.lock() {
            assert_eq!(args.config.model, "moonshotai/kimi-k2:free");
            assert_eq!(args.config.model_provider_id, "openrouter");

            // Most importantly, verify that the model_provider field was updated to match
            assert_eq!(args.config.model_provider.name, "OpenRouter");
            assert_eq!(args.config.model_provider.base_url, Some("https://openrouter.ai/api/v1".to_string()));
            assert_eq!(args.config.model_provider.env_key, Some("OPENROUTER_API_KEY".to_string()));
        }
    }

    #[test]
    fn test_available_models_content() {
        let widget = create_test_widget();
        let models = &widget.available_models;

        // Test that we have the expected models
        assert!(models.iter().any(|m| m.name == "gpt-4o" && m.provider == "OpenAI"));
        assert!(models.iter().any(|m| m.name == "o3" && m.provider == "OpenAI"));
        assert!(models.iter().any(|m| m.name == "openai/gpt-5" && m.provider == "OpenRouter"));
        assert!(models.iter().any(|m| m.name == "anthropic/claude-sonnet-4" && m.provider == "OpenRouter"));
        assert!(models.iter().any(|m| m.name == "grok-4" && m.provider == "xAI"));
        assert!(models.iter().any(|m| m.name == "gpt-oss:20b" && m.provider == "Local OSS"));

        // Test free models
        let free_models: Vec<_> = models.iter().filter(|m| m.is_free).collect();
        assert!(free_models.len() >= 2); // At least qwen and kimi free models

        // Test models requiring API keys
        let api_key_models: Vec<_> = models.iter().filter(|m| m.requires_api_key).collect();
        assert!(api_key_models.len() >= 6); // Most models require API keys
    }
}
