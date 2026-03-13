//! UK Parliament connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    UkParliamentParser, UkMember, UkBill, UkBillStage, UkVote, UkConstituency,
};

/// UK Parliament API connector
///
/// Provides access to UK Parliamentary data including members, bills, and constituencies.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::uk_parliament::UkParliamentConnector;
///
/// let connector = UkParliamentConnector::new();
///
/// // Search for members
/// let members = connector.search_members("Boris", None).await?;
///
/// // Get bill details
/// let bill = connector.get_bill(12345).await?;
///
/// // Search constituencies
/// let constituencies = connector.search_constituencies("London").await?;
/// ```
pub struct UkParliamentConnector {
    client: Client,
    _auth: UkParliamentAuth,
    endpoints: UkParliamentEndpoints,
    _testnet: bool,
}

impl UkParliamentConnector {
    /// Create new UK Parliament connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: UkParliamentAuth::new(),
            endpoints: UkParliamentEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to UK Parliament API
    async fn get(
        &self,
        endpoint: UkParliamentEndpoint,
        params: HashMap<String, String>,
        id: Option<u32>,
    ) -> ExchangeResult<serde_json::Value> {
        let (base_type, base_path) = endpoint.endpoint();
        let base_url = match base_type {
            "members" => self.endpoints.members_base,
            "bills" => self.endpoints.bills_base,
            _ => self.endpoints.members_base,
        };

        let path = if let Some(id_val) = id {
            endpoint.path_with_id(id_val)
        } else {
            base_path.to_string()
        };

        let url = format!("{}{}", base_url, path);

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

        // Check for API errors
        UkParliamentParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // UK PARLIAMENT-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search members by name
    ///
    /// # Arguments
    /// - `name` - Name to search for
    /// - `house` - Optional house filter (1 for Commons, 2 for Lords)
    ///
    /// # Returns
    /// Vector of members matching the search
    pub async fn search_members(
        &self,
        name: &str,
        house: Option<u8>,
    ) -> ExchangeResult<Vec<UkMember>> {
        let mut params = HashMap::new();
        params.insert("Name".to_string(), name.to_string());

        if let Some(h) = house {
            params.insert("House".to_string(), h.to_string());
        }

        let response = self.get(UkParliamentEndpoint::MembersSearch, params, None).await?;
        UkParliamentParser::parse_members(&response)
    }

    /// Get member details by ID
    ///
    /// # Arguments
    /// - `id` - Member ID
    ///
    /// # Returns
    /// Member details
    pub async fn get_member(&self, id: u32) -> ExchangeResult<UkMember> {
        let params = HashMap::new();
        let response = self.get(UkParliamentEndpoint::Member, params, Some(id)).await?;
        UkParliamentParser::parse_member(&response)
    }

    /// Get member voting record
    ///
    /// # Arguments
    /// - `id` - Member ID
    /// - `house` - House (1 for Commons, 2 for Lords)
    ///
    /// # Returns
    /// Vector of voting records
    pub async fn get_member_votes(&self, id: u32, house: u8) -> ExchangeResult<Vec<UkVote>> {
        let mut params = HashMap::new();
        params.insert("House".to_string(), house.to_string());

        let response = self.get(UkParliamentEndpoint::MemberVoting, params, Some(id)).await?;
        UkParliamentParser::parse_votes(&response)
    }

    /// Search bills
    ///
    /// # Arguments
    /// - `query` - Optional search query
    /// - `session` - Optional session ID
    /// - `limit` - Optional limit (default 25)
    ///
    /// # Returns
    /// Vector of bills matching the search
    pub async fn search_bills(
        &self,
        query: Option<&str>,
        session: Option<u32>,
        limit: Option<u16>,
    ) -> ExchangeResult<Vec<UkBill>> {
        let mut params = HashMap::new();

        if let Some(q) = query {
            params.insert("SearchTerm".to_string(), q.to_string());
        }
        if let Some(s) = session {
            params.insert("Session".to_string(), s.to_string());
        }
        if let Some(l) = limit {
            params.insert("Take".to_string(), l.to_string());
        }

        let response = self.get(UkParliamentEndpoint::Bills, params, None).await?;
        UkParliamentParser::parse_bills(&response)
    }

    /// Get bill details by ID
    ///
    /// # Arguments
    /// - `id` - Bill ID
    ///
    /// # Returns
    /// Bill details
    pub async fn get_bill(&self, id: u32) -> ExchangeResult<UkBill> {
        let params = HashMap::new();
        let response = self.get(UkParliamentEndpoint::Bill, params, Some(id)).await?;
        UkParliamentParser::parse_bill(&response)
    }

    /// Get bill stages/progress
    ///
    /// # Arguments
    /// - `id` - Bill ID
    ///
    /// # Returns
    /// Vector of bill stages
    pub async fn get_bill_stages(&self, id: u32) -> ExchangeResult<Vec<UkBillStage>> {
        let params = HashMap::new();
        let response = self.get(UkParliamentEndpoint::BillStages, params, Some(id)).await?;
        UkParliamentParser::parse_bill_stages(&response)
    }

    /// Search constituencies
    ///
    /// # Arguments
    /// - `query` - Search query
    ///
    /// # Returns
    /// Vector of constituencies matching the search
    pub async fn search_constituencies(&self, query: &str) -> ExchangeResult<Vec<UkConstituency>> {
        let mut params = HashMap::new();
        params.insert("searchText".to_string(), query.to_string());

        let response = self.get(UkParliamentEndpoint::ConstituencySearch, params, None).await?;
        UkParliamentParser::parse_constituencies(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get recent bills
    ///
    /// # Arguments
    /// - `limit` - Number of bills to return (default 25)
    ///
    /// # Returns
    /// Vector of recent bills
    pub async fn get_recent_bills(&self, limit: Option<u16>) -> ExchangeResult<Vec<UkBill>> {
        self.search_bills(None, None, limit).await
    }

    /// Get House of Commons members
    ///
    /// # Arguments
    /// - `limit` - Optional limit (use search to get all)
    ///
    /// # Returns
    /// Vector of Commons members
    pub async fn get_commons_members(&self, limit: Option<u16>) -> ExchangeResult<Vec<UkMember>> {
        // Search with empty string returns all members
        let mut members = self.search_members("", Some(1)).await?;

        if let Some(l) = limit {
            members.truncate(l as usize);
        }

        Ok(members)
    }

    /// Get House of Lords members
    ///
    /// # Arguments
    /// - `limit` - Optional limit (use search to get all)
    ///
    /// # Returns
    /// Vector of Lords members
    pub async fn get_lords_members(&self, limit: Option<u16>) -> ExchangeResult<Vec<UkMember>> {
        // Search with empty string returns all members
        let mut members = self.search_members("", Some(2)).await?;

        if let Some(l) = limit {
            members.truncate(l as usize);
        }

        Ok(members)
    }

    /// Ping (check connection)
    pub async fn ping(&self) -> ExchangeResult<()> {
        // Simple ping - try to search for a common name (lightweight check)
        let params = HashMap::new();
        let _ = self.get(UkParliamentEndpoint::MembersSearch, params, None).await?;
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get registered financial interests for a member
    ///
    /// # Arguments
    /// - `member_id` - Member ID
    ///
    /// # Returns
    /// Registered interests as raw JSON value
    pub async fn get_registered_interests(
        &self,
        member_id: u32,
    ) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(UkParliamentEndpoint::RegisteredInterests, params, Some(member_id)).await
    }

    /// Get amendments to a specific bill
    ///
    /// # Arguments
    /// - `bill_id` - Bill ID
    ///
    /// # Returns
    /// Bill amendments as raw JSON value
    pub async fn get_bill_amendments(&self, bill_id: u32) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(UkParliamentEndpoint::AmendmentTracking { bill_id }, params, None).await
    }

    /// Get party composition for each house
    ///
    /// Returns the breakdown of party membership in the House of Commons
    /// and House of Lords.
    ///
    /// # Arguments
    /// - `house` - Optional house filter (1=Commons, 2=Lords)
    ///
    /// # Returns
    /// Party composition data as raw JSON value
    pub async fn get_parties_composition(
        &self,
        house: Option<u8>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        if let Some(h) = house {
            params.insert("House".to_string(), h.to_string());
        }
        self.get(UkParliamentEndpoint::PartiesComposition, params, None).await
    }

    /// Get government posts and ministerial appointments
    ///
    /// # Returns
    /// Government posts as raw JSON value
    pub async fn get_posts(&self) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(UkParliamentEndpoint::Posts, params, None).await
    }
}

impl Default for UkParliamentConnector {
    fn default() -> Self {
        Self::new()
    }
}
