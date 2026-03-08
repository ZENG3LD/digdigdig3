//! Hacker News response parsers
//!
//! Parse JSON responses to domain types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::types::{ExchangeError, ExchangeResult};

pub struct HackerNewsParser;

impl HackerNewsParser {
    /// Parse array of story IDs from JSON
    ///
    /// Example: [39427470, 39426838, 39425902, ...]
    pub fn parse_story_ids(data: &Value) -> ExchangeResult<Vec<u64>> {
        data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of story IDs".to_string()))?
            .iter()
            .map(|v| {
                v.as_u64()
                    .ok_or_else(|| ExchangeError::Parse(format!("Invalid story ID: {:?}", v)))
            })
            .collect()
    }

    /// Parse single story from JSON
    ///
    /// Handles story, ask, and show types (all have similar structure)
    pub fn parse_story(data: &Value) -> ExchangeResult<HnStory> {
        // Handle null response (deleted/not found)
        if data.is_null() {
            return Err(ExchangeError::NotFound("Story not found or deleted".to_string()));
        }

        let obj = data
            .as_object()
            .ok_or_else(|| ExchangeError::Parse("Expected story object".to_string()))?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'id' field".to_string()))?;

        let title = obj
            .get("title")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'title' field".to_string()))?;

        let url = obj
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let score = obj
            .get("score")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let by = obj
            .get("by")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'by' field".to_string()))?;

        let time = obj
            .get("time")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'time' field".to_string()))?;

        let descendants = obj
            .get("descendants")
            .and_then(|v| v.as_u64());

        // Determine item type
        let type_str = obj
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'type' field".to_string()))?;

        let story_type = match type_str {
            "story" => HnItemType::Story,
            "ask" => HnItemType::Story, // Treat ask as story
            "show" => HnItemType::Story, // Treat show as story
            "job" => HnItemType::Job,
            "poll" => HnItemType::Poll,
            "pollopt" => HnItemType::PollOpt,
            "comment" => HnItemType::Comment,
            _ => return Err(ExchangeError::Parse(format!("Unknown item type: {}", type_str))),
        };

        Ok(HnStory {
            id,
            title,
            url,
            score,
            by,
            time,
            descendants,
            story_type,
        })
    }

    /// Parse user from JSON
    pub fn parse_user(data: &Value) -> ExchangeResult<HnUser> {
        // Handle null response (user not found)
        if data.is_null() {
            return Err(ExchangeError::NotFound("User not found".to_string()));
        }

        let obj = data
            .as_object()
            .ok_or_else(|| ExchangeError::Parse("Expected user object".to_string()))?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'id' field".to_string()))?;

        let created = obj
            .get("created")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'created' field".to_string()))?;

        let karma = obj
            .get("karma")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'karma' field".to_string()))?;

        let about = obj
            .get("about")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(HnUser {
            id,
            created,
            karma,
            about,
        })
    }

    /// Parse max item ID from JSON
    pub fn parse_max_item(data: &Value) -> ExchangeResult<u64> {
        data.as_u64()
            .ok_or_else(|| ExchangeError::Parse("Expected u64 for max item ID".to_string()))
    }
}

// =============================================================================
// HACKER NEWS-SPECIFIC TYPES
// =============================================================================

/// Hacker News story metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnStory {
    /// Item ID
    pub id: u64,
    /// Story title
    pub title: String,
    /// Story URL (None for text posts)
    pub url: Option<String>,
    /// Current score (upvotes - downvotes)
    pub score: u64,
    /// Username of author
    pub by: String,
    /// Unix timestamp of creation
    pub time: u64,
    /// Total comment count (recursive)
    pub descendants: Option<u64>,
    /// Item type
    pub story_type: HnItemType,
}

/// Hacker News item type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HnItemType {
    /// Standard story
    Story,
    /// Comment
    Comment,
    /// Job posting
    Job,
    /// Poll
    Poll,
    /// Poll option
    PollOpt,
}

/// Hacker News user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnUser {
    /// Username
    pub id: String,
    /// Account creation timestamp (Unix time)
    pub created: u64,
    /// User karma score
    pub karma: i64,
    /// User bio (optional, may contain HTML)
    pub about: Option<String>,
}

/// Hacker News updates response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnUpdates {
    /// Recently changed item IDs
    pub items: Vec<u64>,
    /// Recently updated usernames
    pub profiles: Vec<String>,
}
