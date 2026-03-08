//! Congress.gov connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    CongressParser, Bill, BillAction, Member, Committee, CongressInfo,
    Nomination, BillCosponsor, BillSummary,
};

/// Congress.gov API connector
///
/// Provides access to legislative data from the United States Congress.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::congress::CongressConnector;
///
/// let connector = CongressConnector::from_env();
///
/// // Get bills from 118th Congress
/// let bills = connector.get_bills(118, None, None).await?;
///
/// // Get specific bill
/// let bill = connector.get_bill(118, "hr", 3076).await?;
///
/// // Get members
/// let members = connector.get_members(None, None).await?;
/// ```
pub struct CongressConnector {
    client: Client,
    auth: CongressAuth,
    endpoints: CongressEndpoints,
    _testnet: bool,
}

impl CongressConnector {
    /// Create new Congress.gov connector with authentication
    pub fn new(auth: CongressAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: CongressEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `CONGRESS_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(CongressAuth::from_env())
    }

    /// Internal: Make GET request to Congress.gov API
    async fn get(
        &self,
        path: &str,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication
        self.auth.sign_query(&mut params);

        // Always request JSON format
        params.insert("format".to_string(), "json".to_string());

        let url = format!("{}{}", self.endpoints.rest_base, path);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for Congress.gov API errors
        CongressParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // BILL ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get bills for a congress
    ///
    /// # Arguments
    /// - `congress` - Congress number (e.g., 118 for 118th Congress)
    /// - `limit` - Optional limit (default 20, max 250)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Returns
    /// Vector of bills
    pub async fn get_bills(
        &self,
        congress: u32,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Bill>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let path = format!("/bill/{}", congress);
        let response = self.get(&path, params).await?;
        CongressParser::parse_bills(&response)
    }

    /// Get specific bill details
    ///
    /// # Arguments
    /// - `congress` - Congress number (e.g., 118)
    /// - `bill_type` - Bill type (hr, s, hjres, sjres, hconres, sconres, hres, sres)
    /// - `number` - Bill number
    ///
    /// # Returns
    /// Bill details
    pub async fn get_bill(
        &self,
        congress: u32,
        bill_type: &str,
        number: u32,
    ) -> ExchangeResult<Bill> {
        let params = HashMap::new();
        let path = format_bill_path(congress, bill_type, number);
        let response = self.get(&path, params).await?;
        CongressParser::parse_bill(&response)
    }

    /// Get bill actions/timeline
    ///
    /// # Arguments
    /// - `congress` - Congress number
    /// - `bill_type` - Bill type
    /// - `number` - Bill number
    ///
    /// # Returns
    /// Vector of bill actions
    pub async fn get_bill_actions(
        &self,
        congress: u32,
        bill_type: &str,
        number: u32,
    ) -> ExchangeResult<Vec<BillAction>> {
        let params = HashMap::new();
        let path = format_bill_actions_path(congress, bill_type, number);
        let response = self.get(&path, params).await?;
        CongressParser::parse_bill_actions(&response)
    }

    /// Get bill cosponsors
    ///
    /// # Arguments
    /// - `congress` - Congress number
    /// - `bill_type` - Bill type
    /// - `number` - Bill number
    ///
    /// # Returns
    /// Vector of bill cosponsors
    pub async fn get_bill_cosponsors(
        &self,
        congress: u32,
        bill_type: &str,
        number: u32,
    ) -> ExchangeResult<Vec<BillCosponsor>> {
        let params = HashMap::new();
        let path = format_bill_cosponsors_path(congress, bill_type, number);
        let response = self.get(&path, params).await?;
        CongressParser::parse_bill_cosponsors(&response)
    }

    /// Get recent bills (convenience method)
    ///
    /// Returns most recent bills from current congress
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default 20, max 250)
    ///
    /// # Returns
    /// Vector of recent bills
    pub async fn get_recent_bills(&self, limit: Option<u32>) -> ExchangeResult<Vec<Bill>> {
        // Current congress is 118 (as of 2024)
        // In production, this should be dynamic
        const CURRENT_CONGRESS: u32 = 118;
        self.get_bills(CURRENT_CONGRESS, limit, None).await
    }

    /// Search bills by policy area (subject)
    ///
    /// # Arguments
    /// - `subject` - Policy area name (e.g., "Commerce", "Health", "Defense")
    /// - `congress` - Congress number
    ///
    /// # Returns
    /// Vector of bills matching the subject
    ///
    /// Note: This is a client-side filter since the API doesn't support subject search directly
    pub async fn get_bills_by_subject(
        &self,
        subject: &str,
        congress: u32,
    ) -> ExchangeResult<Vec<Bill>> {
        // Get all bills for the congress
        let mut all_bills = Vec::new();
        let mut offset = 0;
        const LIMIT: u32 = 250; // max allowed

        loop {
            let bills = self.get_bills(congress, Some(LIMIT), Some(offset)).await?;
            if bills.is_empty() {
                break;
            }

            all_bills.extend(bills);
            offset += LIMIT;

            // Safety limit to avoid infinite loops
            if offset > 10000 {
                break;
            }
        }

        // Filter by policy area
        let subject_lower = subject.to_lowercase();
        let filtered: Vec<Bill> = all_bills
            .into_iter()
            .filter(|bill| {
                bill.policy_area
                    .as_ref()
                    .map(|pa| pa.to_lowercase().contains(&subject_lower))
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered)
    }

    /// Get bill summaries
    ///
    /// # Arguments
    /// - `congress` - Congress number
    /// - `bill_type` - Bill type
    /// - `number` - Bill number
    ///
    /// # Returns
    /// Vector of bill summaries
    pub async fn get_bill_summaries(
        &self,
        congress: u32,
        bill_type: &str,
        number: u32,
    ) -> ExchangeResult<Vec<BillSummary>> {
        let params = HashMap::new();
        let path = format_bill_summaries_path(congress, bill_type, number);
        let response = self.get(&path, params).await?;
        CongressParser::parse_bill_summaries(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MEMBER ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get members of Congress
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default 20, max 250)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Returns
    /// Vector of members
    pub async fn get_members(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<Member>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let path = CongressEndpoint::Members.path();
        let response = self.get(path, params).await?;
        CongressParser::parse_members(&response)
    }

    /// Get specific member by bioguide ID
    ///
    /// # Arguments
    /// - `bioguide_id` - Bioguide ID (e.g., "B000944")
    ///
    /// # Returns
    /// Member details
    pub async fn get_member(&self, bioguide_id: &str) -> ExchangeResult<Member> {
        let params = HashMap::new();
        let path = format_member_path(bioguide_id);
        let response = self.get(&path, params).await?;
        CongressParser::parse_member(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMMITTEE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get committees
    ///
    /// # Arguments
    /// - `chamber` - Optional chamber filter (house, senate, joint)
    /// - `limit` - Optional limit (default 20, max 250)
    ///
    /// # Returns
    /// Vector of committees
    pub async fn get_committees(
        &self,
        chamber: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Committee>> {
        let mut params = HashMap::new();

        if let Some(ch) = chamber {
            params.insert("chamber".to_string(), ch.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let path = CongressEndpoint::Committees.path();
        let response = self.get(path, params).await?;
        CongressParser::parse_committees(&response)
    }

    /// Get specific committee
    ///
    /// # Arguments
    /// - `chamber` - Chamber (house, senate, joint)
    /// - `system_code` - Committee system code
    ///
    /// # Returns
    /// Committee details
    pub async fn get_committee(
        &self,
        chamber: &str,
        system_code: &str,
    ) -> ExchangeResult<Committee> {
        let params = HashMap::new();
        let path = format_committee_path(chamber, system_code);
        let response = self.get(&path, params).await?;
        CongressParser::parse_committee(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NOMINATION ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get nominations for a congress
    ///
    /// # Arguments
    /// - `congress` - Congress number
    /// - `limit` - Optional limit (default 20, max 250)
    ///
    /// # Returns
    /// Vector of nominations
    pub async fn get_nominations(
        &self,
        congress: u32,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Nomination>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let path = format!("/nomination/{}", congress);
        let response = self.get(&path, params).await?;
        CongressParser::parse_nominations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TREATY ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get treaties for a congress
    ///
    /// # Arguments
    /// - `congress` - Congress number
    /// - `limit` - Optional limit (default 20, max 250)
    ///
    /// # Returns
    /// Vector of treaties (parsed as nominations since structure is similar)
    pub async fn get_treaties(
        &self,
        congress: u32,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<Nomination>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let path = format!("/treaty/{}", congress);
        let response = self.get(&path, params).await?;
        // Reuse nominations parser since treaty structure is similar
        CongressParser::parse_nominations(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONGRESS ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of congresses
    ///
    /// # Arguments
    /// - `limit` - Optional limit (default 20, max 250)
    /// - `offset` - Optional offset for pagination
    ///
    /// # Returns
    /// Vector of congress information
    pub async fn get_congresses(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<CongressInfo>> {
        let mut params = HashMap::new();

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let path = CongressEndpoint::Congresses.path();
        let response = self.get(path, params).await?;
        CongressParser::parse_congresses(&response)
    }

    /// Get specific congress information
    ///
    /// # Arguments
    /// - `congress` - Congress number
    ///
    /// # Returns
    /// Congress information
    pub async fn get_congress(&self, congress: u32) -> ExchangeResult<CongressInfo> {
        let params = HashMap::new();
        let path = format_congress_path(congress);
        let response = self.get(&path, params).await?;
        CongressParser::parse_congress(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UTILITY METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Ping (check connection)
    pub async fn ping(&self) -> ExchangeResult<()> {
        // Simple ping - try to get congresses (lightweight endpoint)
        let params = HashMap::new();
        let path = CongressEndpoint::Congresses.path();
        let _ = self.get(path, params).await?;
        Ok(())
    }
}
