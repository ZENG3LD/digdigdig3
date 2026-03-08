//! C2IntelFeeds response parsers
//!
//! Parse CSV responses to domain types.
//!
//! C2IntelFeeds returns CSV format with # prefixed headers. We parse it manually
//! using string operations to extract the key fields.

use crate::core::types::ExchangeResult;

pub struct C2IntelFeedsParser;

/// Indicator type enum
#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorType {
    Ip,
    Domain,
}

/// C2 Indicator structure
#[derive(Debug, Clone)]
pub struct C2Indicator {
    /// IP address (if IP feed)
    pub ip: Option<String>,
    /// Domain name (if domain feed)
    pub domain: Option<String>,
    /// Indicator type
    pub indicator_type: IndicatorType,
    /// IOC classification/description
    pub source: Option<String>,
    /// First seen timestamp (if available)
    pub first_seen: Option<String>,
}

impl C2IntelFeedsParser {
    /// Parse IP feed CSV to indicators
    ///
    /// Expected format:
    /// ```csv
    /// #ip,ioc
    /// 1.12.231.30,Possible Cobaltstrike C2 IP
    /// 1.12.66.17,Possible Cobaltstrike C2 IP
    /// ```
    pub fn parse_ip_feed(csv_text: &str) -> ExchangeResult<Vec<C2Indicator>> {
        let mut indicators = Vec::new();

        for line in csv_text.lines() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Skip comment/header lines (starting with #)
            if line.starts_with('#') {
                continue;
            }

            // Parse CSV line: ip,ioc
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 2 {
                // Skip malformed rows
                continue;
            }

            let ip = parts[0].trim().to_string();
            let ioc = parts[1].trim().to_string();

            indicators.push(C2Indicator {
                ip: Some(ip),
                domain: None,
                indicator_type: IndicatorType::Ip,
                source: Some(ioc),
                first_seen: None,
            });
        }

        Ok(indicators)
    }

    /// Parse domain feed CSV to indicators
    ///
    /// Expected format:
    /// ```csv
    /// #domain,ioc
    /// accesserdsc.com,Possible Cobalt Strike C2 Domain
    /// api.cryptoprot.info,Possible Cobalt Strike C2 Fronting Domain
    /// ```
    pub fn parse_domain_feed(csv_text: &str) -> ExchangeResult<Vec<C2Indicator>> {
        let mut indicators = Vec::new();

        for line in csv_text.lines() {
            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Skip comment/header lines (starting with #)
            if line.starts_with('#') {
                continue;
            }

            // Parse CSV line: domain,ioc
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 2 {
                // Skip malformed rows
                continue;
            }

            let domain = parts[0].trim().to_string();
            let ioc = parts[1].trim().to_string();

            indicators.push(C2Indicator {
                ip: None,
                domain: Some(domain),
                indicator_type: IndicatorType::Domain,
                source: Some(ioc),
                first_seen: None,
            });
        }

        Ok(indicators)
    }
}
