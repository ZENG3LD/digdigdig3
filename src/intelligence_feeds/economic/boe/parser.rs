//! Bank of England response parsers
//!
//! Parse CSV responses to domain types based on BoE API response formats.
//!
//! BoE returns CSV data in the format:
//! ```csv
//! DATE,IUDBEDR
//! 01 Feb 2024,5.25
//! 01 Mar 2024,5.25
//! ```

use crate::core::types::{ExchangeError, ExchangeResult};

pub struct BoeParser;

/// Single observation from BoE time series
#[derive(Debug, Clone)]
pub struct BoeObservation {
    pub date: String,
    pub value: Option<f64>,
}

/// Series information from BoE
#[derive(Debug, Clone)]
pub struct BoeSeriesInfo {
    pub series_code: String,
    pub title: Option<String>,
}

impl BoeParser {
    /// Parse CSV observations from BoE response
    ///
    /// Expected format:
    /// ```csv
    /// DATE,IUDBEDR
    /// 01 Feb 2024,5.25
    /// 01 Mar 2024,5.25
    /// ```
    ///
    /// Multiple series format:
    /// ```csv
    /// DATE,IUDBEDR,LPMAUZI
    /// 01 Feb 2024,5.25,3.2
    /// 01 Mar 2024,5.25,3.4
    /// ```
    pub fn parse_csv_data(csv_text: &str) -> ExchangeResult<Vec<BoeObservation>> {
        let lines: Vec<&str> = csv_text.lines().collect();

        if lines.is_empty() {
            return Err(ExchangeError::Parse("Empty CSV response".to_string()));
        }

        // First line is header: DATE,IUDBEDR or DATE,SERIES1,SERIES2,...
        let header = lines[0];
        let header_parts: Vec<&str> = header.split(',').collect();

        if header_parts.len() < 2 {
            return Err(ExchangeError::Parse("Invalid CSV header".to_string()));
        }

        // For now, we only parse the first series column (index 1)
        // Future enhancement: support multiple series
        let mut observations = Vec::new();

        for line in &lines[1..] {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();

            if parts.len() < 2 {
                continue; // Skip malformed rows
            }

            let date = parts[0].trim().to_string();
            let value_str = parts[1].trim();

            // Parse value - empty or "." means missing data
            let value = if value_str.is_empty() || value_str == "." {
                None
            } else {
                value_str.parse::<f64>().ok()
            };

            observations.push(BoeObservation { date, value });
        }

        Ok(observations)
    }

    /// Parse BoE date format: "DD Mon YYYY" (e.g., "01 Feb 2024")
    /// Returns ISO format: "YYYY-MM-DD"
    pub fn parse_boe_date(date_str: &str) -> ExchangeResult<String> {
        let parts: Vec<&str> = date_str.split_whitespace().collect();

        if parts.len() != 3 {
            return Err(ExchangeError::Parse(format!("Invalid BoE date format: {}", date_str)));
        }

        let day = parts[0];
        let month = Self::parse_month(parts[1])?;
        let year = parts[2];

        Ok(format!("{}-{:02}-{:02}", year, month, day.parse::<u32>()
            .map_err(|_| ExchangeError::Parse(format!("Invalid day: {}", day)))?))
    }

    /// Convert month name to number (1-12)
    fn parse_month(month_str: &str) -> ExchangeResult<u32> {
        let month = match month_str.to_lowercase().as_str() {
            "jan" | "january" => 1,
            "feb" | "february" => 2,
            "mar" | "march" => 3,
            "apr" | "april" => 4,
            "may" => 5,
            "jun" | "june" => 6,
            "jul" | "july" => 7,
            "aug" | "august" => 8,
            "sep" | "september" => 9,
            "oct" | "october" => 10,
            "nov" | "november" => 11,
            "dec" | "december" => 12,
            _ => return Err(ExchangeError::Parse(format!("Invalid month: {}", month_str))),
        };
        Ok(month)
    }

    /// Format date to BoE format: "DD/Mon/YYYY" (e.g., "01/Jan/2020")
    /// Input should be ISO format: "YYYY-MM-DD"
    pub fn format_boe_date(iso_date: &str) -> ExchangeResult<String> {
        let parts: Vec<&str> = iso_date.split('-').collect();

        if parts.len() != 3 {
            return Err(ExchangeError::Parse(format!("Invalid ISO date format: {}", iso_date)));
        }

        let year = parts[0];
        let month_num = parts[1].parse::<u32>()
            .map_err(|_| ExchangeError::Parse(format!("Invalid month: {}", parts[1])))?;
        let day = parts[2].parse::<u32>()
            .map_err(|_| ExchangeError::Parse(format!("Invalid day: {}", parts[2])))?;

        let month_name = Self::format_month(month_num)?;

        Ok(format!("{:02}/{}/{}", day, month_name, year))
    }

    /// Convert month number (1-12) to abbreviated name
    fn format_month(month: u32) -> ExchangeResult<&'static str> {
        let name = match month {
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => return Err(ExchangeError::Parse(format!("Invalid month number: {}", month))),
        };
        Ok(name)
    }

    /// Check for error in BoE response
    /// BoE doesn't return JSON errors, but may return error HTML
    pub fn check_error(response_text: &str) -> ExchangeResult<()> {
        // Check for common error indicators
        if response_text.contains("<html>") || response_text.contains("<!DOCTYPE") {
            return Err(ExchangeError::Api {
                code: 400,
                message: "BoE returned HTML error page".to_string(),
            });
        }

        if response_text.contains("Error") || response_text.contains("error") {
            return Err(ExchangeError::Api {
                code: 400,
                message: format!("BoE API error: {}", response_text.chars().take(200).collect::<String>()),
            });
        }

        Ok(())
    }
}
