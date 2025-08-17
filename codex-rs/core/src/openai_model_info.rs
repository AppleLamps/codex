use crate::model_family::ModelFamily;

/// Metadata about a model, particularly OpenAI models.
/// We may want to consider including details like the pricing for
/// input tokens, output tokens, etc., though users will need to be able to
/// override this in config.toml, as this information can get out of date.
/// Though this would help present more accurate pricing information in the UI.
#[derive(Debug)]
pub(crate) struct ModelInfo {
    /// Size of the context window in tokens.
    pub(crate) context_window: u64,

    /// Maximum number of output tokens that can be generated for the model.
    pub(crate) max_output_tokens: u64,
}

pub(crate) fn get_model_info(model_family: &ModelFamily) -> Option<ModelInfo> {
    let slug = model_family.slug.as_str();
    match slug {
        // OSS models have a 128k shared token pool.
        // Arbitrarily splitting it: 3/4 input context, 1/4 output.
        // https://openai.com/index/gpt-oss-model-card/
        "gpt-oss-20b" => Some(ModelInfo {
            context_window: 96_000,
            max_output_tokens: 32_000,
        }),
        "gpt-oss-120b" => Some(ModelInfo {
            context_window: 96_000,
            max_output_tokens: 32_000,
        }),
        // https://platform.openai.com/docs/models/o3
        "o3" => Some(ModelInfo {
            context_window: 200_000,
            max_output_tokens: 100_000,
        }),

        // https://platform.openai.com/docs/models/o4-mini
        "o4-mini" => Some(ModelInfo {
            context_window: 200_000,
            max_output_tokens: 100_000,
        }),

        // https://platform.openai.com/docs/models/codex-mini-latest
        "codex-mini-latest" => Some(ModelInfo {
            context_window: 200_000,
            max_output_tokens: 100_000,
        }),

        // As of Jun 25, 2025, gpt-4.1 defaults to gpt-4.1-2025-04-14.
        // https://platform.openai.com/docs/models/gpt-4.1
        "gpt-4.1" | "gpt-4.1-2025-04-14" => Some(ModelInfo {
            context_window: 1_047_576,
            max_output_tokens: 32_768,
        }),

        // As of Jun 25, 2025, gpt-4o defaults to gpt-4o-2024-08-06.
        // https://platform.openai.com/docs/models/gpt-4o
        "gpt-4o" | "gpt-4o-2024-08-06" => Some(ModelInfo {
            context_window: 128_000,
            max_output_tokens: 16_384,
        }),

        // https://platform.openai.com/docs/models/gpt-4o?snapshot=gpt-4o-2024-05-13
        "gpt-4o-2024-05-13" => Some(ModelInfo {
            context_window: 128_000,
            max_output_tokens: 4_096,
        }),

        // https://platform.openai.com/docs/models/gpt-4o?snapshot=gpt-4o-2024-11-20
        "gpt-4o-2024-11-20" => Some(ModelInfo {
            context_window: 128_000,
            max_output_tokens: 16_384,
        }),

        // https://platform.openai.com/docs/models/gpt-3.5-turbo
        "gpt-3.5-turbo" => Some(ModelInfo {
            context_window: 16_385,
            max_output_tokens: 4_096,
        }),

        "grok-4" => Some(ModelInfo {
            context_window: 256_000,
            max_output_tokens: 256_000,
        }),

        "gpt-5" => Some(ModelInfo {
            context_window: 200_000,
            max_output_tokens: 100_000,
        }),

        // OpenRouter models with accurate specifications
        "openai/gpt-5" => Some(ModelInfo {
            context_window: 400_000,
            max_output_tokens: 180_000,
        }),

        "anthropic/claude-sonnet-4" => Some(ModelInfo {
            context_window: 200_000,
            max_output_tokens: 64_000,
        }),

        "moonshotai/kimi-k2:free" => Some(ModelInfo {
            context_window: 65_500,
            max_output_tokens: 65_500,
        }),

        "qwen/qwen3-coder:free" => Some(ModelInfo {
            context_window: 262_144,
            max_output_tokens: 262_144,
        }),

        _ if slug.starts_with("codex-") => Some(ModelInfo {
            context_window: 200_000,
            max_output_tokens: 100_000,
        }),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_model_info() {
        use crate::model_family::find_family_for_model;

        // Test OpenAI GPT-5 via OpenRouter
        let gpt5_family = find_family_for_model("openai/gpt-5").expect("openai/gpt-5 should be recognized");
        let gpt5_info = get_model_info(&gpt5_family).expect("openai/gpt-5 should have model info");
        assert_eq!(gpt5_info.context_window, 400_000);
        assert_eq!(gpt5_info.max_output_tokens, 180_000);

        // Test Anthropic Claude Sonnet 4 via OpenRouter
        let claude_family = find_family_for_model("anthropic/claude-sonnet-4").expect("anthropic/claude-sonnet-4 should be recognized");
        let claude_info = get_model_info(&claude_family).expect("anthropic/claude-sonnet-4 should have model info");
        assert_eq!(claude_info.context_window, 200_000);
        assert_eq!(claude_info.max_output_tokens, 64_000);

        // Test Moonshot Kimi K2 via OpenRouter
        let kimi_family = find_family_for_model("moonshotai/kimi-k2:free").expect("moonshotai/kimi-k2:free should be recognized");
        let kimi_info = get_model_info(&kimi_family).expect("moonshotai/kimi-k2:free should have model info");
        assert_eq!(kimi_info.context_window, 65_500);
        assert_eq!(kimi_info.max_output_tokens, 65_500);

        // Test Qwen 3 Coder via OpenRouter
        let qwen_family = find_family_for_model("qwen/qwen3-coder:free").expect("qwen/qwen3-coder:free should be recognized");
        let qwen_info = get_model_info(&qwen_family).expect("qwen/qwen3-coder:free should have model info");
        assert_eq!(qwen_info.context_window, 262_144);
        assert_eq!(qwen_info.max_output_tokens, 262_144);
    }

    #[test]
    fn test_openrouter_models_have_large_context_windows() {
        use crate::model_family::find_family_for_model;

        // Verify that OpenRouter models have substantial context windows
        let models = [
            ("openai/gpt-5", 400_000),
            ("anthropic/claude-sonnet-4", 200_000),
            ("moonshotai/kimi-k2:free", 65_500),
            ("qwen/qwen3-coder:free", 262_144),
        ];

        for (model, expected_context) in models {
            let family = find_family_for_model(model).expect(&format!("{} should be recognized", model));
            let info = get_model_info(&family).expect(&format!("{} should have model info", model));
            assert_eq!(info.context_window, expected_context, "Context window mismatch for {}", model);
            assert!(info.context_window >= 65_000, "{} should have at least 65k context window", model);
        }
    }

    #[test]
    fn test_openrouter_models_have_substantial_output_limits() {
        use crate::model_family::find_family_for_model;

        // Verify that OpenRouter models have good output token limits
        let models = [
            ("openai/gpt-5", 180_000),
            ("anthropic/claude-sonnet-4", 64_000),
            ("moonshotai/kimi-k2:free", 65_500),
            ("qwen/qwen3-coder:free", 262_144),
        ];

        for (model, expected_output) in models {
            let family = find_family_for_model(model).expect(&format!("{} should be recognized", model));
            let info = get_model_info(&family).expect(&format!("{} should have model info", model));
            assert_eq!(info.max_output_tokens, expected_output, "Output token limit mismatch for {}", model);
            assert!(info.max_output_tokens >= 64_000, "{} should have at least 64k output tokens", model);
        }
    }

    #[test]
    fn test_existing_models_still_work() {
        use crate::model_family::find_family_for_model;

        // Ensure we didn't break existing model info
        let gpt4o_family = find_family_for_model("gpt-4o").expect("gpt-4o should be recognized");
        assert!(get_model_info(&gpt4o_family).is_some());

        let o3_family = find_family_for_model("o3").expect("o3 should be recognized");
        assert!(get_model_info(&o3_family).is_some());

        let gpt5_family = find_family_for_model("gpt-5").expect("gpt-5 should be recognized");
        assert!(get_model_info(&gpt5_family).is_some());
    }

    #[test]
    fn test_unknown_models_return_none() {
        use crate::model_family::find_family_for_model;

        // Unknown models should not be recognized at the family level
        assert!(find_family_for_model("unknown/model").is_none());
        assert!(find_family_for_model("openrouter/unknown").is_none());
    }
}
