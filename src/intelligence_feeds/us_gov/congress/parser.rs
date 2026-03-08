//! Congress.gov response parsers
//!
//! Parse JSON responses to domain types based on Congress.gov API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct CongressParser;

impl CongressParser {
    // ═══════════════════════════════════════════════════════════════════════
    // CONGRESS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse bills list
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "bills": [
    ///     {
    ///       "congress": 118,
    ///       "type": "hr",
    ///       "number": "3076",
    ///       "title": "Example Bill",
    ///       "policyArea": {"name": "Commerce"},
    ///       "latestAction": {"text": "Referred to committee", "actionDate": "2024-01-15"},
    ///       "updateDate": "2024-01-15T12:00:00Z",
    ///       "originChamber": "House",
    ///       "introducedDate": "2024-01-10"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_bills(response: &Value) -> ExchangeResult<Vec<Bill>> {
        let bills = response
            .get("bills")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'bills' array".to_string()))?;

        bills
            .iter()
            .map(|bill| {
                Ok(Bill {
                    congress: Self::get_u32(bill, "congress"),
                    type_code: Self::get_str(bill, "type").map(|s| s.to_string()),
                    number: Self::get_str(bill, "number").map(|s| s.to_string()),
                    title: Self::get_str(bill, "title").map(|s| s.to_string()),
                    policy_area: bill
                        .get("policyArea")
                        .and_then(|pa| pa.get("name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    latest_action: bill
                        .get("latestAction")
                        .and_then(|la| la.get("text"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    latest_action_date: bill
                        .get("latestAction")
                        .and_then(|la| la.get("actionDate"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    update_date: Self::get_str(bill, "updateDate").map(|s| s.to_string()),
                    origin_chamber: Self::get_str(bill, "originChamber").map(|s| s.to_string()),
                    introduced_date: Self::get_str(bill, "introducedDate").map(|s| s.to_string()),
                    url: Self::get_str(bill, "url").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single bill
    pub fn parse_bill(response: &Value) -> ExchangeResult<Bill> {
        let bill = response
            .get("bill")
            .ok_or_else(|| ExchangeError::Parse("Missing 'bill' object".to_string()))?;

        Ok(Bill {
            congress: Self::get_u32(bill, "congress"),
            type_code: Self::get_str(bill, "type").map(|s| s.to_string()),
            number: Self::get_str(bill, "number").map(|s| s.to_string()),
            title: Self::get_str(bill, "title").map(|s| s.to_string()),
            policy_area: bill
                .get("policyArea")
                .and_then(|pa| pa.get("name"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            latest_action: bill
                .get("latestAction")
                .and_then(|la| la.get("text"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            latest_action_date: bill
                .get("latestAction")
                .and_then(|la| la.get("actionDate"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            update_date: Self::get_str(bill, "updateDate").map(|s| s.to_string()),
            origin_chamber: Self::get_str(bill, "originChamber").map(|s| s.to_string()),
            introduced_date: Self::get_str(bill, "introducedDate").map(|s| s.to_string()),
            url: Self::get_str(bill, "url").map(|s| s.to_string()),
        })
    }

    /// Parse bill actions
    pub fn parse_bill_actions(response: &Value) -> ExchangeResult<Vec<BillAction>> {
        let actions = response
            .get("actions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'actions' array".to_string()))?;

        actions
            .iter()
            .map(|action| {
                Ok(BillAction {
                    action_date: Self::get_str(action, "actionDate").map(|s| s.to_string()),
                    text: Self::get_str(action, "text").map(|s| s.to_string()),
                    action_type: Self::get_str(action, "type").map(|s| s.to_string()),
                    action_code: Self::get_str(action, "actionCode").map(|s| s.to_string()),
                    source_system: action
                        .get("sourceSystem")
                        .and_then(|ss| ss.get("name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    committee: action
                        .get("committees")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|c| c.get("name"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse bill cosponsors
    pub fn parse_bill_cosponsors(response: &Value) -> ExchangeResult<Vec<BillCosponsor>> {
        let cosponsors = response
            .get("cosponsors")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'cosponsors' array".to_string()))?;

        cosponsors
            .iter()
            .map(|cosponsor| {
                Ok(BillCosponsor {
                    bioguide_id: Self::get_str(cosponsor, "bioguideId").map(|s| s.to_string()),
                    full_name: Self::get_str(cosponsor, "fullName").map(|s| s.to_string()),
                    party: Self::get_str(cosponsor, "party").map(|s| s.to_string()),
                    state: Self::get_str(cosponsor, "state").map(|s| s.to_string()),
                    district: Self::get_u32(cosponsor, "district"),
                    sponsorship_date: Self::get_str(cosponsor, "sponsorshipDate").map(|s| s.to_string()),
                    is_original_cosponsor: Self::get_bool(cosponsor, "isOriginalCosponsor"),
                })
            })
            .collect()
    }

    /// Parse members list
    pub fn parse_members(response: &Value) -> ExchangeResult<Vec<Member>> {
        let members = response
            .get("members")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'members' array".to_string()))?;

        members
            .iter()
            .map(|member| {
                Ok(Member {
                    bioguide_id: Self::get_str(member, "bioguideId").map(|s| s.to_string()),
                    name: Self::get_str(member, "name").map(|s| s.to_string()),
                    party: Self::get_str(member, "partyName").map(|s| s.to_string()),
                    state: Self::get_str(member, "state").map(|s| s.to_string()),
                    district: Self::get_u32(member, "district"),
                    chamber: member
                        .get("terms")
                        .and_then(|v| v.get("item"))
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|t| t.get("chamber"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    url: Self::get_str(member, "url").map(|s| s.to_string()),
                    served_since: member
                        .get("terms")
                        .and_then(|v| v.get("item"))
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|t| t.get("startYear"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single member
    pub fn parse_member(response: &Value) -> ExchangeResult<Member> {
        let member = response
            .get("member")
            .ok_or_else(|| ExchangeError::Parse("Missing 'member' object".to_string()))?;

        Ok(Member {
            bioguide_id: Self::get_str(member, "bioguideId").map(|s| s.to_string()),
            name: Self::get_str(member, "name").map(|s| s.to_string()),
            party: Self::get_str(member, "partyName").map(|s| s.to_string()),
            state: Self::get_str(member, "state").map(|s| s.to_string()),
            district: Self::get_u32(member, "district"),
            chamber: member
                .get("terms")
                .and_then(|v| v.get("item"))
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("chamber"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            url: Self::get_str(member, "url").map(|s| s.to_string()),
            served_since: member
                .get("terms")
                .and_then(|v| v.get("item"))
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|t| t.get("startYear"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// Parse committees list
    pub fn parse_committees(response: &Value) -> ExchangeResult<Vec<Committee>> {
        let committees = response
            .get("committees")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'committees' array".to_string()))?;

        committees
            .iter()
            .map(|committee| {
                Ok(Committee {
                    system_code: Self::get_str(committee, "systemCode").map(|s| s.to_string()),
                    name: Self::get_str(committee, "name").map(|s| s.to_string()),
                    chamber: Self::get_str(committee, "chamber").map(|s| s.to_string()),
                    committee_type: Self::get_str(committee, "type").map(|s| s.to_string()),
                    url: Self::get_str(committee, "url").map(|s| s.to_string()),
                    parent: committee
                        .get("parent")
                        .and_then(|p| p.get("systemCode"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single committee
    pub fn parse_committee(response: &Value) -> ExchangeResult<Committee> {
        let committee = response
            .get("committee")
            .ok_or_else(|| ExchangeError::Parse("Missing 'committee' object".to_string()))?;

        Ok(Committee {
            system_code: Self::get_str(committee, "systemCode").map(|s| s.to_string()),
            name: Self::get_str(committee, "name").map(|s| s.to_string()),
            chamber: Self::get_str(committee, "chamber").map(|s| s.to_string()),
            committee_type: Self::get_str(committee, "type").map(|s| s.to_string()),
            url: Self::get_str(committee, "url").map(|s| s.to_string()),
            parent: committee
                .get("parent")
                .and_then(|p| p.get("systemCode"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }

    /// Parse congresses list
    pub fn parse_congresses(response: &Value) -> ExchangeResult<Vec<CongressInfo>> {
        let congresses = response
            .get("congresses")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'congresses' array".to_string()))?;

        congresses
            .iter()
            .map(|congress| {
                Ok(CongressInfo {
                    number: Self::get_u32(congress, "number"),
                    name: Self::get_str(congress, "name").map(|s| s.to_string()),
                    start_year: Self::get_str(congress, "startYear").map(|s| s.to_string()),
                    end_year: Self::get_str(congress, "endYear").map(|s| s.to_string()),
                    sessions: congress
                        .get("sessions")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.len() as u32),
                    url: Self::get_str(congress, "url").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single congress
    pub fn parse_congress(response: &Value) -> ExchangeResult<CongressInfo> {
        let congress = response
            .get("congress")
            .ok_or_else(|| ExchangeError::Parse("Missing 'congress' object".to_string()))?;

        Ok(CongressInfo {
            number: Self::get_u32(congress, "number"),
            name: Self::get_str(congress, "name").map(|s| s.to_string()),
            start_year: Self::get_str(congress, "startYear").map(|s| s.to_string()),
            end_year: Self::get_str(congress, "endYear").map(|s| s.to_string()),
            sessions: congress
                .get("sessions")
                .and_then(|v| v.as_array())
                .map(|arr| arr.len() as u32),
            url: Self::get_str(congress, "url").map(|s| s.to_string()),
        })
    }

    /// Parse nominations list
    pub fn parse_nominations(response: &Value) -> ExchangeResult<Vec<Nomination>> {
        let nominations = response
            .get("nominations")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'nominations' array".to_string()))?;

        nominations
            .iter()
            .map(|nomination| {
                Ok(Nomination {
                    congress: Self::get_u32(nomination, "congress"),
                    number: Self::get_str(nomination, "number").map(|s| s.to_string()),
                    part_number: Self::get_str(nomination, "partNumber").map(|s| s.to_string()),
                    description: Self::get_str(nomination, "description").map(|s| s.to_string()),
                    received_date: Self::get_str(nomination, "receivedDate").map(|s| s.to_string()),
                    latest_action: nomination
                        .get("latestAction")
                        .and_then(|la| la.get("text"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    latest_action_date: nomination
                        .get("latestAction")
                        .and_then(|la| la.get("actionDate"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    url: Self::get_str(nomination, "url").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse bill summaries
    pub fn parse_bill_summaries(response: &Value) -> ExchangeResult<Vec<BillSummary>> {
        let summaries = response
            .get("summaries")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'summaries' array".to_string()))?;

        summaries
            .iter()
            .map(|summary| {
                Ok(BillSummary {
                    action_date: Self::get_str(summary, "actionDate").map(|s| s.to_string()),
                    action_desc: Self::get_str(summary, "actionDesc").map(|s| s.to_string()),
                    text: Self::get_str(summary, "text").map(|s| s.to_string()),
                    update_date: Self::get_str(summary, "updateDate").map(|s| s.to_string()),
                    version_code: Self::get_str(summary, "versionCode").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            let code = error
                .get("code")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

            return Err(ExchangeError::Api { code, message });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn _require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field)
            .and_then(|v| {
                v.as_u64().map(|n| n as u32)
                    .or_else(|| v.as_str().and_then(|s| s.parse::<u32>().ok()))
            })
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CONGRESS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Congressional bill
#[derive(Debug, Clone)]
pub struct Bill {
    pub congress: Option<u32>,
    pub type_code: Option<String>,
    pub number: Option<String>,
    pub title: Option<String>,
    pub policy_area: Option<String>,
    pub latest_action: Option<String>,
    pub latest_action_date: Option<String>,
    pub update_date: Option<String>,
    pub origin_chamber: Option<String>,
    pub introduced_date: Option<String>,
    pub url: Option<String>,
}

/// Bill action
#[derive(Debug, Clone)]
pub struct BillAction {
    pub action_date: Option<String>,
    pub text: Option<String>,
    pub action_type: Option<String>,
    pub action_code: Option<String>,
    pub source_system: Option<String>,
    pub committee: Option<String>,
}

/// Bill cosponsor
#[derive(Debug, Clone)]
pub struct BillCosponsor {
    pub bioguide_id: Option<String>,
    pub full_name: Option<String>,
    pub party: Option<String>,
    pub state: Option<String>,
    pub district: Option<u32>,
    pub sponsorship_date: Option<String>,
    pub is_original_cosponsor: Option<bool>,
}

/// Congressional member
#[derive(Debug, Clone)]
pub struct Member {
    pub bioguide_id: Option<String>,
    pub name: Option<String>,
    pub party: Option<String>,
    pub state: Option<String>,
    pub district: Option<u32>,
    pub chamber: Option<String>,
    pub url: Option<String>,
    pub served_since: Option<String>,
}

/// Congressional committee
#[derive(Debug, Clone)]
pub struct Committee {
    pub system_code: Option<String>,
    pub name: Option<String>,
    pub chamber: Option<String>,
    pub committee_type: Option<String>,
    pub url: Option<String>,
    pub parent: Option<String>,
}

/// Congress information
#[derive(Debug, Clone)]
pub struct CongressInfo {
    pub number: Option<u32>,
    pub name: Option<String>,
    pub start_year: Option<String>,
    pub end_year: Option<String>,
    pub sessions: Option<u32>,
    pub url: Option<String>,
}

/// Nomination
#[derive(Debug, Clone)]
pub struct Nomination {
    pub congress: Option<u32>,
    pub number: Option<String>,
    pub part_number: Option<String>,
    pub description: Option<String>,
    pub received_date: Option<String>,
    pub latest_action: Option<String>,
    pub latest_action_date: Option<String>,
    pub url: Option<String>,
}

/// Bill summary
#[derive(Debug, Clone)]
pub struct BillSummary {
    pub action_date: Option<String>,
    pub action_desc: Option<String>,
    pub text: Option<String>,
    pub update_date: Option<String>,
    pub version_code: Option<String>,
}
