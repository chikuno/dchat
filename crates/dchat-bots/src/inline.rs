//! Inline query handling for bots

use dchat_core::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Inline query from user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineQuery {
    /// Query ID
    pub id: Uuid,
    
    /// User who made the query
    pub from: dchat_core::types::UserId,
    
    /// Query text
    pub query: String,
    
    /// Offset for pagination
    pub offset: String,
    
    /// Chat type (if sent from chat)
    pub chat_type: Option<ChatType>,
    
    /// User location (if shared)
    pub location: Option<Location>,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Chat type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatType {
    Private,
    Group,
    Supergroup,
    Channel,
}

/// Geographic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

/// Inline query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineResult {
    /// Result type
    pub result_type: InlineResultType,
    
    /// Unique result ID
    pub id: String,
    
    /// Title
    pub title: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Thumbnail URL
    pub thumbnail_url: Option<String>,
    
    /// Content
    pub content: InlineContent,
}

/// Type of inline result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InlineResultType {
    Article,
    Photo,
    Video,
    Audio,
    Document,
    Location,
    Venue,
    Contact,
}

/// Inline content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InlineContent {
    Text {
        text: String,
        parse_mode: Option<crate::ParseMode>,
    },
    Photo {
        photo_url: String,
        caption: Option<String>,
    },
    Video {
        video_url: String,
        mime_type: String,
        caption: Option<String>,
    },
    Audio {
        audio_url: String,
        title: String,
        performer: Option<String>,
    },
    Document {
        document_url: String,
        mime_type: String,
        caption: Option<String>,
    },
    Location {
        latitude: f64,
        longitude: f64,
        title: String,
    },
}

/// Answer inline query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerInlineQueryRequest {
    /// Inline query ID
    pub inline_query_id: Uuid,
    
    /// Results to show
    pub results: Vec<InlineResult>,
    
    /// Cache time in seconds
    pub cache_time: Option<u32>,
    
    /// Is result personal (not cached)?
    pub is_personal: bool,
    
    /// Next offset for pagination
    pub next_offset: Option<String>,
}

/// Inline query handler trait
pub trait InlineQueryHandler {
    /// Handle an inline query
    fn handle(&self, query: &InlineQuery) -> Result<Vec<InlineResult>>;
}

/// Simple text inline query handler
pub struct TextInlineQueryHandler {
    #[allow(dead_code)]
    bot_name: String,
}

impl TextInlineQueryHandler {
    pub fn new(bot_name: String) -> Self {
        Self { bot_name }
    }
}

impl InlineQueryHandler for TextInlineQueryHandler {
    fn handle(&self, query: &InlineQuery) -> Result<Vec<InlineResult>> {
        if query.query.is_empty() {
            return Ok(vec![
                InlineResult {
                    result_type: InlineResultType::Article,
                    id: "help".to_string(),
                    title: "Type something...".to_string(),
                    description: Some("Start typing to search".to_string()),
                    thumbnail_url: None,
                    content: InlineContent::Text {
                        text: "Please type something to search".to_string(),
                        parse_mode: None,
                    },
                },
            ]);
        }
        
        // Simple echo example
        let results = vec![
            InlineResult {
                result_type: InlineResultType::Article,
                id: "echo".to_string(),
                title: format!("Echo: {}", query.query),
                description: Some("Send this message".to_string()),
                thumbnail_url: None,
                content: InlineContent::Text {
                    text: query.query.clone(),
                    parse_mode: None,
                },
            },
            InlineResult {
                result_type: InlineResultType::Article,
                id: "markdown".to_string(),
                title: format!("**{}**", query.query),
                description: Some("Send as markdown".to_string()),
                thumbnail_url: None,
                content: InlineContent::Text {
                    text: format!("**{}**", query.query),
                    parse_mode: Some(crate::ParseMode::Markdown),
                },
            },
        ];
        
        Ok(results)
    }
}

/// Image search inline query handler
pub struct ImageSearchHandler;

impl InlineQueryHandler for ImageSearchHandler {
    fn handle(&self, query: &InlineQuery) -> Result<Vec<InlineResult>> {
        // Mock image search results
        let results = (1..=5)
            .map(|i| InlineResult {
                result_type: InlineResultType::Photo,
                id: format!("photo_{}", i),
                title: format!("Photo {} - {}", i, query.query),
                description: Some(format!("Result {}", i)),
                thumbnail_url: Some(format!("https://placekitten.com/200/200?image={}", i)),
                content: InlineContent::Photo {
                    photo_url: format!("https://placekitten.com/800/600?image={}", i),
                    caption: Some(query.query.clone()),
                },
            })
            .collect();
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dchat_core::types::UserId;
    
    #[test]
    fn test_inline_query() {
        let query = InlineQuery {
            id: Uuid::new_v4(),
            from: UserId::new(),
            query: "test search".to_string(),
            offset: String::new(),
            chat_type: Some(ChatType::Private),
            location: None,
            timestamp: Utc::now(),
        };
        
        assert_eq!(query.query, "test search");
    }
    
    #[test]
    fn test_text_inline_handler_empty() {
        let handler = TextInlineQueryHandler::new("testbot".to_string());
        let query = InlineQuery {
            id: Uuid::new_v4(),
            from: UserId::new(),
            query: String::new(),
            offset: String::new(),
            chat_type: None,
            location: None,
            timestamp: Utc::now(),
        };
        
        let results = handler.handle(&query).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "help");
    }
    
    #[test]
    fn test_text_inline_handler_with_query() {
        let handler = TextInlineQueryHandler::new("testbot".to_string());
        let query = InlineQuery {
            id: Uuid::new_v4(),
            from: UserId::new(),
            query: "hello world".to_string(),
            offset: String::new(),
            chat_type: None,
            location: None,
            timestamp: Utc::now(),
        };
        
        let results = handler.handle(&query).unwrap();
        assert_eq!(results.len(), 2); // echo and markdown
        assert_eq!(results[0].id, "echo");
        assert_eq!(results[1].id, "markdown");
    }
    
    #[test]
    fn test_image_search_handler() {
        let handler = ImageSearchHandler;
        let query = InlineQuery {
            id: Uuid::new_v4(),
            from: UserId::new(),
            query: "cats".to_string(),
            offset: String::new(),
            chat_type: None,
            location: None,
            timestamp: Utc::now(),
        };
        
        let results = handler.handle(&query).unwrap();
        assert_eq!(results.len(), 5);
        
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.id, format!("photo_{}", i + 1));
            assert!(matches!(result.result_type, InlineResultType::Photo));
        }
    }
}
