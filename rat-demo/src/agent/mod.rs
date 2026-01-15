pub mod gomoku;
pub mod commentator;

pub use gomoku::GomokuAgent;
pub use commentator::CommentatorAgent;
use rig::providers::openai;

/// Helper to create an OpenAI-backed Gomoku agent.
pub fn create_gomoku_openai_agent(api_key: &str, model: &str, base_url: Option<&str>) -> anyhow::Result<GomokuAgent<openai::CompletionModel>> {
    use rig::client::CompletionClient;
    use rig::providers::openai::CompletionsClient;

    let client = match base_url {
        Some(url) => CompletionsClient::<reqwest::Client>::builder().api_key(api_key).base_url(url).build(),
        None => CompletionsClient::<reqwest::Client>::builder().api_key(api_key).build(),
    }.map_err(|e| anyhow::anyhow!("Failed to initialize OpenAI client: {}", e))?;
    
    let completion_model = client.completion_model(model);
    Ok(GomokuAgent::new(completion_model))
}

/// Creates a Gomoku agent using environment variables:
/// - `GOMOKU_API_KEY`: Required API key
/// - `GOMOKU_MODEL`: Required model name (e.g., gpt-4o)
/// - `GOMOKU_API_BASE`: Optional custom base URL for OpenAI-compatible APIs
pub fn create_gomoku_agent_from_env() -> anyhow::Result<GomokuAgent<openai::CompletionModel>> {
    let api_key = std::env::var("GOMOKU_API_KEY")
        .map_err(|_| anyhow::anyhow!("GOMOKU_API_KEY environment variable not set"))?;
    let model = std::env::var("GOMOKU_MODEL")
        .map_err(|_| anyhow::anyhow!("GOMOKU_MODEL environment variable not set"))?;
    let base_url = std::env::var("GOMOKU_API_BASE").ok();

    create_gomoku_openai_agent(&api_key, &model, base_url.as_deref())
}

/// Creates a Commentator agent using environment variables.
pub fn create_commentator_agent_from_env() -> anyhow::Result<CommentatorAgent> {
    let api_key = std::env::var("GOMOKU_API_KEY")
        .map_err(|_| anyhow::anyhow!("GOMOKU_API_KEY (used for commentator) not set"))?;
    let model = std::env::var("GOMOKU_MODEL")
        .map_err(|_| anyhow::anyhow!("GOMOKU_MODEL (used for commentator) not set"))?;
    let base_url = std::env::var("GOMOKU_API_BASE").ok();

    use rig::providers::openai::CompletionsClient;
    use rig::client::CompletionClient;
    let client = match base_url {
        Some(url) => CompletionsClient::<reqwest::Client>::builder().api_key(api_key).base_url(url).build(),
        None => CompletionsClient::<reqwest::Client>::builder().api_key(api_key).build(),
    }.map_err(|e| anyhow::anyhow!("Failed to initialize commentator client: {}", e))?;
    
    let completion_model = client.completion_model(model);
    Ok(CommentatorAgent::new(completion_model))
}
