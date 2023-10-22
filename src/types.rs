use super::*;

fn deserialize_maybe_null<'de, D>(deserializer: D) -> Result<String, D::Error>
    where D: Deserializer<'de> {
    let buf = Option::<String>::deserialize(deserializer)?;
    Ok(buf.unwrap_or(String::new()))
}

/// A response struct received from the API after requesting a message completion
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize,Default)]
pub struct CompletionResponse {
    /// Unique ID of the message, but not in a UUID format.
    /// Example: `chatcmpl-6p5FEv1JHictSSnDZsGU4KvbuBsbu`
    #[serde(rename = "id")]
    pub message_id: Option<String>,
    /// Unix seconds timestamp of when the response was created
    #[serde(rename = "created")]
    pub created_timestamp: Option<u64>,
    /// The model that was used for this completion
    pub model: String,
    /// Token usage of this completion
    pub usage: TokenUsage,
    /// Message choices for this response, guaranteed to contain at least one message response
    #[serde(rename = "choices")]
    pub message_choices: Vec<MessageChoice>,
}
/// A message completion choice struct
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize)]
pub struct MessageChoice {
    /// The actual message
    pub message: ChatMessage,
    /// The reason completion was stopped
    pub finish_reason: String,
    /// The index of this message in the outer `message_choices` array
    pub index: u32,
}
/// A role of a message sender, can be:
/// - `System`, for starting system message, that sets the tone of model
/// - `Assistant`, for messages sent by ChatGPT
/// - `User`, for messages sent by user
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Eq, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// A system message, automatically sent at the start to set the tone of the model
    System,
    /// A message sent by ChatGPT
    Assistant,
    /// A message sent by the user
    User,
    /// A message related to ChatGPT functions. Does not have much use without the `functions` feature.
    Function,
}

/// Container for the sent/received ChatGPT messages
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Role of message sender
    pub role: Role,
    /// Actual content of the message
    #[serde(deserialize_with = "deserialize_maybe_null")]
    pub content: String,
    /// Function call (if present)
    #[cfg(feature = "functions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
}

/// The token usage of a specific response
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize,Default)]
pub struct TokenUsage {
    /// Tokens spent on the prompt message (including previous messages)
    pub prompt_tokens: u32,
    /// Tokens spent on the completion message
    pub completion_tokens: u32,
    /// Total amount of tokens used (`prompt_tokens + completion_tokens`)
    pub total_tokens: u32,
}