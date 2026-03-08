//! UK Parliament response parsers
//!
//! Parse JSON responses to domain types based on UK Parliament API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct UkParliamentParser;

impl UkParliamentParser {
    // ═══════════════════════════════════════════════════════════════════════
    // UK PARLIAMENT-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse members search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "items": [...],
    ///   "totalResults": 100
    /// }
    /// ```
    pub fn parse_members(response: &Value) -> ExchangeResult<Vec<UkMember>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|member| {
                let member_id = Self::require_u32(member, "id")?;
                let name = Self::get_str(member, "nameDisplayAs")
                    .or_else(|| Self::get_str(member, "nameFullTitle"))
                    .unwrap_or("Unknown")
                    .to_string();

                Ok(UkMember {
                    member_id,
                    name,
                    party: Self::get_str(member, "latestParty")
                        .and_then(|p| {
                            serde_json::from_value::<Value>(serde_json::json!(p))
                                .ok()
                                .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                        })
                        .or_else(|| Self::get_str(member, "partyName").map(|s| s.to_string())),
                    constituency: Self::get_str(member, "latestHouseMembership")
                        .and_then(|m| {
                            serde_json::from_value::<Value>(serde_json::json!(m))
                                .ok()
                                .and_then(|v| v.get("membershipFrom").and_then(|n| n.as_str()).map(|s| s.to_string()))
                        })
                        .or_else(|| Self::get_str(member, "constituency").map(|s| s.to_string())),
                    house: Self::parse_house(member),
                    gender: Self::get_str(member, "gender").map(|s| s.to_string()),
                    membership_start: Self::get_str(member, "membershipStartDate")
                        .or_else(|| member.get("latestHouseMembership")
                            .and_then(|v| v.get("membershipStartDate"))
                            .and_then(|d| d.as_str()))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse member details
    pub fn parse_member(response: &Value) -> ExchangeResult<UkMember> {
        let value = response
            .get("value")
            .unwrap_or(response);

        let member_id = Self::require_u32(value, "id")?;
        let name = Self::get_str(value, "nameDisplayAs")
            .or_else(|| Self::get_str(value, "nameFullTitle"))
            .unwrap_or("Unknown")
            .to_string();

        Ok(UkMember {
            member_id,
            name,
            party: Self::get_str(value, "latestParty")
                .and_then(|p| {
                    serde_json::from_value::<Value>(serde_json::json!(p))
                        .ok()
                        .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                }),
            constituency: Self::get_str(value, "latestHouseMembership")
                .and_then(|m| {
                    serde_json::from_value::<Value>(serde_json::json!(m))
                        .ok()
                        .and_then(|v| v.get("membershipFrom").and_then(|n| n.as_str()).map(|s| s.to_string()))
                }),
            house: Self::parse_house(value),
            gender: Self::get_str(value, "gender").map(|s| s.to_string()),
            membership_start: Self::get_str(value, "membershipStartDate")
                .or_else(|| value.get("latestHouseMembership")
                    .and_then(|v| v.get("membershipStartDate"))
                    .and_then(|d| d.as_str()))
                .map(|s| s.to_string()),
        })
    }

    /// Parse voting records
    pub fn parse_votes(response: &Value) -> ExchangeResult<Vec<UkVote>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|vote| {
                let division_id = Self::require_u32(vote, "divisionId")
                    .or_else(|_| Self::require_u32(vote, "id"))?;

                Ok(UkVote {
                    division_id,
                    division_date: Self::get_str(vote, "divisionDate")
                        .or_else(|| Self::get_str(vote, "date"))
                        .unwrap_or("")
                        .to_string(),
                    division_title: Self::get_str(vote, "title")
                        .or_else(|| Self::get_str(vote, "divisionTitle"))
                        .unwrap_or("")
                        .to_string(),
                    member_voted_aye: Self::get_str(vote, "memberVotedAye")
                        .and_then(|s| s.parse::<bool>().ok())
                        .or_else(|| Self::get_bool(vote, "memberVotedAye"))
                        .unwrap_or(false),
                })
            })
            .collect()
    }

    /// Parse bills search results
    pub fn parse_bills(response: &Value) -> ExchangeResult<Vec<UkBill>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|bill| {
                Ok(UkBill {
                    bill_id: Self::require_u32(bill, "billId")?,
                    short_title: Self::get_str(bill, "shortTitle")
                        .unwrap_or("")
                        .to_string(),
                    long_title: Self::get_str(bill, "longTitle").map(|s| s.to_string()),
                    current_house: Self::get_str(bill, "currentHouse")
                        .or_else(|| Self::get_str(bill, "originatingHouse"))
                        .map(|s| s.to_string()),
                    bill_type: Self::get_str(bill, "billTypeDescription")
                        .or_else(|| Self::get_str(bill, "billType"))
                        .map(|s| s.to_string()),
                    is_act: Self::get_bool(bill, "isAct").unwrap_or(false),
                    sessions: Self::parse_sessions(bill),
                })
            })
            .collect()
    }

    /// Parse bill details
    pub fn parse_bill(response: &Value) -> ExchangeResult<UkBill> {
        let bill = response;

        Ok(UkBill {
            bill_id: Self::require_u32(bill, "billId")?,
            short_title: Self::get_str(bill, "shortTitle")
                .unwrap_or("")
                .to_string(),
            long_title: Self::get_str(bill, "longTitle").map(|s| s.to_string()),
            current_house: Self::get_str(bill, "currentHouse")
                .or_else(|| Self::get_str(bill, "originatingHouse"))
                .map(|s| s.to_string()),
            bill_type: Self::get_str(bill, "billTypeDescription")
                .or_else(|| Self::get_str(bill, "billType"))
                .map(|s| s.to_string()),
            is_act: Self::get_bool(bill, "isAct").unwrap_or(false),
            sessions: Self::parse_sessions(bill),
        })
    }

    /// Parse bill stages
    pub fn parse_bill_stages(response: &Value) -> ExchangeResult<Vec<UkBillStage>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|stage| {
                Ok(UkBillStage {
                    stage_id: Self::require_u32(stage, "id")
                        .or_else(|_| Self::require_u32(stage, "stageId"))?,
                    description: Self::get_str(stage, "description")
                        .or_else(|| Self::get_str(stage, "stageName"))
                        .unwrap_or("")
                        .to_string(),
                    house: Self::get_str(stage, "house")
                        .or_else(|| Self::get_str(stage, "houseName"))
                        .map(|s| s.to_string()),
                    sitting_date: Self::get_str(stage, "sittingDate")
                        .or_else(|| Self::get_str(stage, "date"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse constituencies search results
    pub fn parse_constituencies(response: &Value) -> ExchangeResult<Vec<UkConstituency>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|constituency| {
                Ok(UkConstituency {
                    constituency_id: Self::require_u32(constituency, "id")?,
                    name: Self::get_str(constituency, "name")
                        .or_else(|| Self::get_str(constituency, "constituencyName"))
                        .unwrap_or("")
                        .to_string(),
                    current_representation: Self::get_str(constituency, "currentRepresentation")
                        .and_then(|r| {
                            serde_json::from_value::<Value>(serde_json::json!(r))
                                .ok()
                                .and_then(|v| v.get("member").and_then(|m| m.get("nameDisplayAs")).and_then(|n| n.as_str()).map(|s| s.to_string()))
                        })
                        .or_else(|| Self::get_str(constituency, "memberName").map(|s| s.to_string())),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn parse_house(obj: &Value) -> Option<String> {
        Self::get_str(obj, "latestHouseMembership")
            .and_then(|m| {
                serde_json::from_value::<Value>(serde_json::json!(m))
                    .ok()
                    .and_then(|v| v.get("house").and_then(|h| h.as_str()).map(|s| s.to_string()))
            })
            .or_else(|| Self::get_str(obj, "house").map(|s| s.to_string()))
    }

    fn parse_sessions(bill: &Value) -> Option<String> {
        bill.get("sessions")
            .and_then(|sessions| {
                if let Some(arr) = sessions.as_array() {
                    let session_names: Vec<String> = arr
                        .iter()
                        .filter_map(|s| s.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                        .collect();
                    if !session_names.is_empty() {
                        Some(session_names.join(", "))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }

    fn require_u32(obj: &Value, field: &str) -> ExchangeResult<u32> {
        obj.get(field)
            .and_then(|v| {
                if let Some(num) = v.as_u64() {
                    Some(num as u32)
                } else if let Some(s) = v.as_str() {
                    s.parse::<u32>().ok()
                } else {
                    None
                }
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UK PARLIAMENT-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// UK Parliament member (MP or Lord)
#[derive(Debug, Clone)]
pub struct UkMember {
    pub member_id: u32,
    pub name: String,
    pub party: Option<String>,
    pub constituency: Option<String>,
    pub house: Option<String>, // "Commons" or "Lords"
    pub gender: Option<String>,
    pub membership_start: Option<String>,
}

/// UK Parliamentary bill
#[derive(Debug, Clone)]
pub struct UkBill {
    pub bill_id: u32,
    pub short_title: String,
    pub long_title: Option<String>,
    pub current_house: Option<String>,
    pub bill_type: Option<String>,
    pub is_act: bool,
    pub sessions: Option<String>,
}

/// UK Bill stage
#[derive(Debug, Clone)]
pub struct UkBillStage {
    pub stage_id: u32,
    pub description: String,
    pub house: Option<String>,
    pub sitting_date: Option<String>,
}

/// UK Parliamentary vote
#[derive(Debug, Clone)]
pub struct UkVote {
    pub division_id: u32,
    pub division_date: String,
    pub division_title: String,
    pub member_voted_aye: bool,
}

/// UK Constituency
#[derive(Debug, Clone)]
pub struct UkConstituency {
    pub constituency_id: u32,
    pub name: String,
    pub current_representation: Option<String>,
}
