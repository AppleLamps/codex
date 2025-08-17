/// A model family is a group of models that share certain characteristics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelFamily {
    /// The full model slug used to derive this model family, e.g.
    /// "gpt-4.1-2025-04-14".
    pub slug: String,

    /// The model family name, e.g. "gpt-4.1". Note this should able to be used
    /// with [`crate::openai_model_info::get_model_info`].
    pub family: String,

    /// True if the model needs additional instructions on how to use the
    /// "virtual" `apply_patch` CLI.
    pub needs_special_apply_patch_instructions: bool,

    // Whether the `reasoning` field can be set when making a request to this
    // model family. Note it has `effort` and `summary` subfields (though
    // `summary` is optional).
    pub supports_reasoning_summaries: bool,

    // This should be set to true when the model expects a tool named
    // "local_shell" to be provided. Its contract must be understood natively by
    // the model such that its description can be omitted.
    // See https://platform.openai.com/docs/guides/tools-local-shell
    pub uses_local_shell_tool: bool,

    /// True if the model performs better when `apply_patch` is provided as
    /// a tool call instead of just a bash command.
    pub uses_apply_patch_tool: bool,
}

macro_rules! model_family {
    (
        $slug:expr, $family:expr $(, $key:ident : $value:expr )* $(,)?
    ) => {{
        // defaults
        let mut mf = ModelFamily {
            slug: $slug.to_string(),
            family: $family.to_string(),
            needs_special_apply_patch_instructions: false,
            supports_reasoning_summaries: false,
            uses_local_shell_tool: false,
            uses_apply_patch_tool: false,
        };
        // apply overrides
        $(
            mf.$key = $value;
        )*
        Some(mf)
    }};
}

macro_rules! simple_model_family {
    (
        $slug:expr, $family:expr
    ) => {{
        Some(ModelFamily {
            slug: $slug.to_string(),
            family: $family.to_string(),
            needs_special_apply_patch_instructions: false,
            supports_reasoning_summaries: false,
            uses_local_shell_tool: false,
            uses_apply_patch_tool: false,
        })
    }};
}

/// Returns a `ModelFamily` for the given model slug, or `None` if the slug
/// does not match any known model family.
pub fn find_family_for_model(slug: &str) -> Option<ModelFamily> {
    if slug.starts_with("o3") {
        model_family!(
            slug, "o3",
            supports_reasoning_summaries: true,
        )
    } else if slug.starts_with("o4-mini") {
        model_family!(
            slug, "o4-mini",
            supports_reasoning_summaries: true,
        )
    } else if slug.starts_with("codex-mini-latest") {
        model_family!(
            slug, "codex-mini-latest",
            supports_reasoning_summaries: true,
            uses_local_shell_tool: true,
        )
    } else if slug.starts_with("codex-") {
        model_family!(
            slug, slug,
            supports_reasoning_summaries: true,
        )
    } else if slug.starts_with("gpt-4.1") {
        model_family!(
            slug, "gpt-4.1",
            needs_special_apply_patch_instructions: true,
        )
    } else if slug.starts_with("gpt-oss") {
        model_family!(slug, "gpt-oss", uses_apply_patch_tool: true)
    } else if slug.starts_with("gpt-4o") {
        simple_model_family!(slug, "gpt-4o")
    } else if slug.starts_with("gpt-3.5") {
        simple_model_family!(slug, "gpt-3.5")
    } else if slug.starts_with("gpt-5") {
        model_family!(
            slug, "gpt-5",
            supports_reasoning_summaries: true,
        )
    } else if slug.starts_with("grok-4") {
        model_family!(
            slug, "grok-4",
            supports_reasoning_summaries: true,
        )
    } else if slug.starts_with("openai/gpt-5") {
        model_family!(
            slug, "openai/gpt-5",
            supports_reasoning_summaries: true,
        )
    } else if slug.starts_with("anthropic/claude-sonnet-4") {
        model_family!(
            slug, "anthropic/claude-sonnet-4",
            supports_reasoning_summaries: true,
        )
    } else if slug.starts_with("moonshotai/kimi-k2") {
        simple_model_family!(slug, "moonshotai/kimi-k2")
    } else if slug.starts_with("qwen/qwen3-coder") {
        simple_model_family!(slug, "qwen/qwen3-coder")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_model_families() {
        // Test OpenAI GPT-5 via OpenRouter
        let gpt5_family = find_family_for_model("openai/gpt-5").expect("openai/gpt-5 should be recognized");
        assert_eq!(gpt5_family.slug, "openai/gpt-5");
        assert_eq!(gpt5_family.family, "openai/gpt-5");
        assert_eq!(gpt5_family.supports_reasoning_summaries, true);
        assert_eq!(gpt5_family.needs_special_apply_patch_instructions, false);
        assert_eq!(gpt5_family.uses_local_shell_tool, false);
        assert_eq!(gpt5_family.uses_apply_patch_tool, false);

        // Test Anthropic Claude Sonnet 4 via OpenRouter
        let claude_family = find_family_for_model("anthropic/claude-sonnet-4").expect("anthropic/claude-sonnet-4 should be recognized");
        assert_eq!(claude_family.slug, "anthropic/claude-sonnet-4");
        assert_eq!(claude_family.family, "anthropic/claude-sonnet-4");
        assert_eq!(claude_family.supports_reasoning_summaries, true);
        assert_eq!(claude_family.needs_special_apply_patch_instructions, false);
        assert_eq!(claude_family.uses_local_shell_tool, false);
        assert_eq!(claude_family.uses_apply_patch_tool, false);

        // Test Moonshot Kimi K2 via OpenRouter
        let kimi_family = find_family_for_model("moonshotai/kimi-k2:free").expect("moonshotai/kimi-k2:free should be recognized");
        assert_eq!(kimi_family.slug, "moonshotai/kimi-k2:free");
        assert_eq!(kimi_family.family, "moonshotai/kimi-k2");
        assert_eq!(kimi_family.supports_reasoning_summaries, false);
        assert_eq!(kimi_family.needs_special_apply_patch_instructions, false);
        assert_eq!(kimi_family.uses_local_shell_tool, false);
        assert_eq!(kimi_family.uses_apply_patch_tool, false);

        // Test Qwen 3 Coder via OpenRouter
        let qwen_family = find_family_for_model("qwen/qwen3-coder:free").expect("qwen/qwen3-coder:free should be recognized");
        assert_eq!(qwen_family.slug, "qwen/qwen3-coder:free");
        assert_eq!(qwen_family.family, "qwen/qwen3-coder");
        assert_eq!(qwen_family.supports_reasoning_summaries, false);
        assert_eq!(qwen_family.needs_special_apply_patch_instructions, false);
        assert_eq!(qwen_family.uses_local_shell_tool, false);
        assert_eq!(qwen_family.uses_apply_patch_tool, false);
    }

    #[test]
    fn test_openrouter_model_family_prefixes() {
        // Test that different versions/variants are recognized
        assert!(find_family_for_model("openai/gpt-5-turbo").is_some());
        assert!(find_family_for_model("anthropic/claude-sonnet-4-20241120").is_some());
        assert!(find_family_for_model("moonshotai/kimi-k2").is_some());
        assert!(find_family_for_model("qwen/qwen3-coder").is_some());
    }

    #[test]
    fn test_unknown_openrouter_models() {
        // Test that unknown models return None
        assert!(find_family_for_model("unknown/model").is_none());
        assert!(find_family_for_model("openrouter/unknown").is_none());
    }

    #[test]
    fn test_existing_model_families_still_work() {
        // Ensure we didn't break existing model families
        assert!(find_family_for_model("gpt-4o").is_some());
        assert!(find_family_for_model("o3").is_some());
        assert!(find_family_for_model("grok-4").is_some());
        assert!(find_family_for_model("gpt-5").is_some());
    }
}
