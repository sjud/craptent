use std::collections::HashMap;
use super::*;



#[derive(Props,PartialEq)]
pub struct MessageChoicesProps{
  pub choices:Vec<MessageChoice>,
}
#[derive(Debug,Clone,PartialEq,Default)]
pub struct AppState{
  pub current_record:Option<StringRecord>,
  pub chat_gpt_system_raw:String,
  pub chat_gpt_system_edited:String,
  pub chat_gpt_prompt_raw:String,
  pub chat_gpt_prompt_edited:String,
  pub dall_e_raw:String,
  pub dall_e_edited:String,
  pub eleven_labs_raw:String,
  pub eleven_labs_edited:String,
}
pub enum AppStateFieldUpdate{
  ChatGPTSystem(String),
  ChatGPTPrompt(String),
  DallE(String),
  ElevenLabs(String),
}
impl AppState{
  pub fn update_field(&mut self, update:AppStateFieldUpdate) {
    match update {
        AppStateFieldUpdate::ChatGPTSystem(s) => self.chat_gpt_system_raw=s,
        AppStateFieldUpdate::ChatGPTPrompt(s) => self.chat_gpt_prompt_raw=s,
        AppStateFieldUpdate::DallE(s) => self.dall_e_raw=s,
        AppStateFieldUpdate::ElevenLabs(s) => self.eleven_labs_raw=s,
    }
    if let Some(current_record) = &self.current_record {
      self.update_current_record(current_record.clone());
    } else {
      self.chat_gpt_system_edited = self.chat_gpt_system_raw.clone();
      self.chat_gpt_prompt_edited = self.chat_gpt_prompt_raw.clone();
      self.eleven_labs_edited = self.eleven_labs_raw.clone();
      self.dall_e_edited = self.dall_e_raw.clone();
    }
  }
  pub fn update_current_record(&mut self, record:StringRecord) {
    let mut i = 0;
    let mut chat_gpt_system_edited = self.chat_gpt_system_raw.clone();
    let mut chat_gpt_prompt_edited = self.chat_gpt_prompt_raw.clone();
    let mut dall_e_edited = self.dall_e_raw.clone();
    let mut eleven_labs_edited = self.eleven_labs_raw.clone();
    while record.get(i).is_some() {
      let s = record.get(i).unwrap();
      chat_gpt_system_edited = chat_gpt_system_edited.replace(&format!("{{{}}}",i),s);
      chat_gpt_prompt_edited = chat_gpt_prompt_edited.replace(&format!("{{{}}}",i),s);
      eleven_labs_edited = eleven_labs_edited.replace(&format!("{{{}}}",i),s);
      dall_e_edited = dall_e_edited.replace(&format!("{{{}}}",i),s);
      i+=1;
    }
    self.chat_gpt_system_edited = chat_gpt_system_edited;
    self.chat_gpt_prompt_edited = chat_gpt_prompt_edited;
    self.eleven_labs_edited = eleven_labs_edited;
    self.dall_e_edited = dall_e_edited;
    self.current_record=Some(record);
  }
}

#[derive(Default,Clone,Debug,PartialEq)]
pub struct ApiKeys{
    pub open_ai:String,
    pub eleven_labs:String,
}
#[derive(Props,PartialEq)]
pub struct ApiKeyProps{
    pub model:GenModel,
}
#[derive(Clone,Debug,Copy,PartialEq)]
pub enum GenModel{
    OpenAI,
    ElevenLabs,
}

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

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize,Default)]
pub struct DallEResponse{
    /// Unix seconds timestamp of when the response was created
    #[serde(rename = "created")]
    pub created_timestamp: Option<u64>,
    /// The list of image urls.
    pub data: Vec<ImageObject>,
}   
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize,Default)]
pub struct ImageObject{
    /// The Url of the Image generated by Dall-E
    pub url:String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct VoicesResponse{
    pub voices:Vec<Voice>,
}



#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, )]
pub struct Voice {
    pub voice_id: String,
    pub name: String,
    pub samples: Option<Vec<VoiceSample>>,
    pub category: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub preview_url: Option<String>,
    pub settings: Option<VoiceSettings>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct VoiceSample {
    pub sample_id: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: Option<i64>,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct VoiceSettings {
    pub similarity_boost: f64,
    pub stability: f64,
    pub style: f64,
    pub use_speaker_boost: bool,
}

/*

	

Successful Response
Media type
Controls Accept header.

{
  "voices": [
    {
      "voice_id": "string",
      "name": "string",
      "samples": [
        {
          "sample_id": "string",
          "file_name": "string",
          "mime_type": "string",
          "size_bytes": 0,
          "hash": "string"
        }
      ],
      "category": "string",
      "fine_tuning": {
        "language": "string",
        "is_allowed_to_fine_tune": true,
        "fine_tuning_requested": true,
        "finetuning_state": "not_started",
        "verification_attempts": [
          {
            "text": "string",
            "date_unix": 0,
            "accepted": true,
            "similarity": 0,
            "levenshtein_distance": 0,
            "recording": {
              "recording_id": "string",
              "mime_type": "string",
              "size_bytes": 0,
              "upload_date_unix": 0,
              "transcription": "string"
            }
          }
        ],
        "verification_failures": [
          "string"
        ],
        "verification_attempts_count": 0,
        "slice_ids": [
          "string"
        ],
        "manual_verification": {
          "extra_text": "string",
          "request_time_unix": 0,
          "files": [
            {
              "file_id": "string",
              "file_name": "string",
              "mime_type": "string",
              "size_bytes": 0,
              "upload_date_unix": 0
            }
          ]
        },
        "manual_verification_requested": true
      },
      "labels": {
        "additionalProp1": "string",
        "additionalProp2": "string",
        "additionalProp3": "string"
      },
      "description": "string",
      "preview_url": "string",
      "available_for_tiers": [
        "string"
      ],
      "settings": {
        "stability": 0,
        "similarity_boost": 0,
        "style": 0,
        "use_speaker_boost": true
      },
      "sharing": {
        "status": "enabled",
        "history_item_sample_id": "string",
        "original_voice_id": "string",
        "public_owner_id": "string",
        "liked_by_count": 0,
        "cloned_by_count": 0,
        "whitelisted_emails": [
          "string"
        ],
        "name": "string",
        "labels": {
          "additionalProp1": "string",
          "additionalProp2": "string",
          "additionalProp3": "string"
        },
        "description": "string",
        "review_status": "not_requested",
        "review_message": "string",
        "enabled_in_library": true
      },
      "high_quality_base_model_ids": [
        "string"
      ]
    }
  ]
}*/