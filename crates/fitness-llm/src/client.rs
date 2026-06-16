use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, ResponseFormat,
        ResponseFormatJsonSchema,
    },
};
use fitness_core::config::LlmConfig;
use serde_json::Value;

pub struct LlmClient {
    client: Client<OpenAIConfig>,
    model: String,
    max_tokens: u32,
}

impl LlmClient {
    pub fn new(config: &LlmConfig) -> Self {
        let openai_config = OpenAIConfig::default()
            .with_api_key(&config.api_key)
            .with_api_base(&config.api_base);

        let client = Client::with_config(openai_config);

        Self {
            client,
            model: config.model.clone(),
            max_tokens: config.max_tokens,
        }
    }

    pub async fn chat(
        &self,
        system_prompt: &str,
        user_message: &str,
    ) -> Result<String, fitness_core::error::AppError> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt.to_string())
                .build()
                .map(|m| ChatCompletionRequestMessage::System(m))
                .map_err(|e| fitness_core::error::AppError::LlmError(e.to_string()))?,
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_message.to_string())
                .build()
                .map(|m| ChatCompletionRequestMessage::User(m))
                .map_err(|e| fitness_core::error::AppError::LlmError(e.to_string()))?,
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .max_tokens(self.max_tokens)
            .messages(messages)
            .build()
            .map_err(|e: async_openai::error::OpenAIError| {
                fitness_core::error::AppError::LlmError(e.to_string())
            })?;

        let response = self.client.chat().create(request).await.map_err(
            |e: async_openai::error::OpenAIError| {
                fitness_core::error::AppError::LlmError(e.to_string())
            },
        )?;

        response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| fitness_core::error::AppError::LlmError("No response from LLM".into()))
    }

    pub async fn chat_with_json_schema(
        &self,
        system_prompt: &str,
        user_message: &str,
        schema_name: &str,
        schema: Value,
    ) -> Result<String, fitness_core::error::AppError> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt.to_string())
                .build()
                .map(|m| ChatCompletionRequestMessage::System(m))
                .map_err(|e| fitness_core::error::AppError::LlmError(e.to_string()))?,
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_message.to_string())
                .build()
                .map(|m| ChatCompletionRequestMessage::User(m))
                .map_err(|e| fitness_core::error::AppError::LlmError(e.to_string()))?,
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .max_tokens(self.max_tokens)
            .messages(messages)
            .response_format(ResponseFormat::JsonSchema {
                json_schema: ResponseFormatJsonSchema {
                    name: schema_name.to_string(),
                    description: None,
                    schema: Some(schema),
                    strict: Some(true),
                },
            })
            .build()
            .map_err(|e: async_openai::error::OpenAIError| {
                fitness_core::error::AppError::LlmError(e.to_string())
            })?;

        let response = self.client.chat().create(request).await.map_err(
            |e: async_openai::error::OpenAIError| {
                fitness_core::error::AppError::LlmError(e.to_string())
            },
        )?;

        response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| fitness_core::error::AppError::LlmError("No response from LLM".into()))
    }
}
