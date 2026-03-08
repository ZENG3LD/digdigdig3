//! FAA Airport Status response parsers
//!
//! Parse XML responses to domain types.
//!
//! FAA returns XML format. We parse it manually using simple string operations
//! to extract the key fields we need.

use crate::core::types::ExchangeResult;

pub struct FaaStatusParser;

impl FaaStatusParser {
    /// Parse FAA Airport Status XML response
    ///
    /// Example response structure:
    /// ```xml
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <AIRPORT_STATUS_INFORMATION>
    ///   <Update_Time>Mon Feb 16 09:01:29 2026 GMT</Update_Time>
    ///   <Dtd_File>http://nasstatus.faa.gov/xml/airport_status.dtd</Dtd_File>
    ///   <Delay_type>
    ///     <Name>Airport Closures</Name>
    ///     <Airport_Closure_List>
    ///       <Airport>
    ///         <ARPT>MMU</ARPT>
    ///         <Reason><![CDATA[MMU 02/014 MMU AD AP CLSD TO ALL ACFT 2602160900-2602161500]]></Reason>
    ///         <Start>2602160900</Start>
    ///         <Reopen>2602161500</Reopen>
    ///       </Airport>
    ///     </Airport_Closure_List>
    ///   </Delay_type>
    /// </AIRPORT_STATUS_INFORMATION>
    /// ```
    pub fn parse_airport_status(xml: &str) -> ExchangeResult<AirportStatus> {
        // Extract timestamp
        let timestamp = Self::extract_tag_content(xml, "Update_Time")
            .unwrap_or_else(|| "Unknown".to_string());

        let mut delays = Vec::new();

        // Find all Delay_type sections
        let mut search_start = 0;
        while let Some(pos) = xml[search_start..].find("<Delay_type>") {
            let delay_start = search_start + pos;
            if let Some(delay_end) = xml[delay_start..].find("</Delay_type>") {
                let delay_block = &xml[delay_start..delay_start + delay_end];

                // Extract delay type name
                let delay_type_name = Self::extract_tag_content(delay_block, "Name")
                    .unwrap_or_else(|| "Unknown".to_string());

                // Parse based on delay type
                match delay_type_name.as_str() {
                    "Airport Closures" => {
                        delays.extend(Self::parse_airport_closures(delay_block));
                    }
                    "Ground Delay Programs" => {
                        delays.extend(Self::parse_ground_delays(delay_block));
                    }
                    "Ground Stops" => {
                        delays.extend(Self::parse_ground_stops(delay_block));
                    }
                    "Arrival/Departure Delay Info" => {
                        delays.extend(Self::parse_arrival_departure_delays(delay_block));
                    }
                    "Airspace Flow Programs" => {
                        delays.extend(Self::parse_airspace_flow_programs(delay_block));
                    }
                    _ => {}
                }

                search_start = delay_start + delay_end + "</Delay_type>".len();
            } else {
                break;
            }
        }

        let count = delays.len();
        Ok(AirportStatus {
            timestamp,
            delays,
            count,
        })
    }

    /// Parse airport closures
    fn parse_airport_closures(xml: &str) -> Vec<AirportDelay> {
        let mut delays = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = xml[search_start..].find("<Airport>") {
            let airport_start = search_start + pos;
            if let Some(airport_end) = xml[airport_start..].find("</Airport>") {
                let airport_block = &xml[airport_start..airport_start + airport_end];

                if let Some(code) = Self::extract_tag_content(airport_block, "ARPT") {
                    let reason = Self::extract_cdata_content(airport_block, "Reason");
                    let end_time = Self::extract_tag_content(airport_block, "Reopen");

                    delays.push(AirportDelay {
                        airport_code: code,
                        airport_name: None,
                        city: None,
                        state: None,
                        delay_type: DelayType::Closure,
                        severity: DelaySeverity::Major,
                        avg_delay_minutes: None,
                        reason,
                        end_time,
                        last_updated: None,
                    });
                }

                search_start = airport_start + airport_end + "</Airport>".len();
            } else {
                break;
            }
        }

        delays
    }

    /// Parse ground delay programs
    fn parse_ground_delays(xml: &str) -> Vec<AirportDelay> {
        let mut delays = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = xml[search_start..].find("<Ground_Delay>") {
            let delay_start = search_start + pos;
            if let Some(delay_end) = xml[delay_start..].find("</Ground_Delay>") {
                let delay_block = &xml[delay_start..delay_start + delay_end];

                if let Some(code) = Self::extract_tag_content(delay_block, "ARPT") {
                    let reason = Self::extract_tag_content(delay_block, "Reason");
                    let avg_delay = Self::extract_tag_content(delay_block, "Avg_Delay")
                        .and_then(|s| s.parse::<i32>().ok());
                    let end_time = Self::extract_tag_content(delay_block, "End");

                    // Determine severity based on delay minutes
                    let severity = match avg_delay {
                        Some(d) if d >= 60 => DelaySeverity::Severe,
                        Some(d) if d >= 45 => DelaySeverity::Major,
                        Some(d) if d >= 30 => DelaySeverity::Moderate,
                        Some(d) if d >= 15 => DelaySeverity::Minor,
                        _ => DelaySeverity::Normal,
                    };

                    delays.push(AirportDelay {
                        airport_code: code,
                        airport_name: None,
                        city: None,
                        state: None,
                        delay_type: DelayType::GroundDelay,
                        severity,
                        avg_delay_minutes: avg_delay,
                        reason,
                        end_time,
                        last_updated: None,
                    });
                }

                search_start = delay_start + delay_end + "</Ground_Delay>".len();
            } else {
                break;
            }
        }

        delays
    }

    /// Parse ground stops
    fn parse_ground_stops(xml: &str) -> Vec<AirportDelay> {
        let mut delays = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = xml[search_start..].find("<Ground_Stop>") {
            let stop_start = search_start + pos;
            if let Some(stop_end) = xml[stop_start..].find("</Ground_Stop>") {
                let stop_block = &xml[stop_start..stop_start + stop_end];

                if let Some(code) = Self::extract_tag_content(stop_block, "ARPT") {
                    let reason = Self::extract_tag_content(stop_block, "Reason");
                    let end_time = Self::extract_tag_content(stop_block, "End_Time");

                    delays.push(AirportDelay {
                        airport_code: code,
                        airport_name: None,
                        city: None,
                        state: None,
                        delay_type: DelayType::GroundStop,
                        severity: DelaySeverity::Severe,
                        avg_delay_minutes: None,
                        reason,
                        end_time,
                        last_updated: None,
                    });
                }

                search_start = stop_start + stop_end + "</Ground_Stop>".len();
            } else {
                break;
            }
        }

        delays
    }

    /// Parse arrival/departure delays
    fn parse_arrival_departure_delays(xml: &str) -> Vec<AirportDelay> {
        let mut delays = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = xml[search_start..].find("<Delay>") {
            let delay_start = search_start + pos;
            if let Some(delay_end) = xml[delay_start..].find("</Delay>") {
                let delay_block = &xml[delay_start..delay_start + delay_end];

                if let Some(code) = Self::extract_tag_content(delay_block, "ARPT") {
                    let reason = Self::extract_tag_content(delay_block, "Reason");

                    // Extract arrival delay info
                    let arrival_max = if let Some(arrival_block) = Self::extract_block(delay_block, "Arrival_Delay") {
                        Self::extract_tag_content(&arrival_block, "Max")
                            .and_then(|s| s.parse::<i32>().ok())
                    } else {
                        None
                    };

                    // Extract departure delay info
                    let departure_max = if let Some(departure_block) = Self::extract_block(delay_block, "Departure_Delay") {
                        Self::extract_tag_content(&departure_block, "Max")
                            .and_then(|s| s.parse::<i32>().ok())
                    } else {
                        None
                    };

                    // Use the maximum of arrival/departure delays
                    let avg_delay = match (arrival_max, departure_max) {
                        (Some(a), Some(d)) => Some(a.max(d)),
                        (Some(a), None) => Some(a),
                        (None, Some(d)) => Some(d),
                        (None, None) => None,
                    };

                    // Determine severity
                    let severity = match avg_delay {
                        Some(d) if d >= 45 => DelaySeverity::Major,
                        Some(d) if d >= 30 => DelaySeverity::Moderate,
                        Some(d) if d >= 15 => DelaySeverity::Minor,
                        _ => DelaySeverity::Normal,
                    };

                    delays.push(AirportDelay {
                        airport_code: code,
                        airport_name: None,
                        city: None,
                        state: None,
                        delay_type: DelayType::ArrivalDelay,
                        severity,
                        avg_delay_minutes: avg_delay,
                        reason,
                        end_time: None,
                        last_updated: None,
                    });
                }

                search_start = delay_start + delay_end + "</Delay>".len();
            } else {
                break;
            }
        }

        delays
    }

    /// Parse airspace flow programs
    fn parse_airspace_flow_programs(xml: &str) -> Vec<AirportDelay> {
        let mut delays = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = xml[search_start..].find("<Program>") {
            let program_start = search_start + pos;
            if let Some(program_end) = xml[program_start..].find("</Program>") {
                let program_block = &xml[program_start..program_start + program_end];

                let reason = Self::extract_tag_content(program_block, "Reason");
                let end_time = Self::extract_tag_content(program_block, "End");

                // Extract affected airports
                let affected_airports = Self::extract_all_airports(program_block);

                // Create a delay entry for each affected airport
                for code in affected_airports {
                    delays.push(AirportDelay {
                        airport_code: code,
                        airport_name: None,
                        city: None,
                        state: None,
                        delay_type: DelayType::AirspaceFlowProgram,
                        severity: DelaySeverity::Moderate,
                        avg_delay_minutes: None,
                        reason: reason.clone(),
                        end_time: end_time.clone(),
                        last_updated: None,
                    });
                }

                search_start = program_start + program_end + "</Program>".len();
            } else {
                break;
            }
        }

        delays
    }

    /// Extract content between opening and closing tags
    fn extract_tag_content(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        let start = text.find(&open_tag)? + open_tag.len();
        let end = text[start..].find(&close_tag)? + start;

        Some(text[start..end].trim().to_string())
    }

    /// Extract CDATA content (for Reason fields)
    fn extract_cdata_content(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        let tag_start = text.find(&open_tag)? + open_tag.len();
        let tag_end = text[tag_start..].find(&close_tag)? + tag_start;
        let tag_content = &text[tag_start..tag_end];

        // Check for CDATA
        if let Some(cdata_start) = tag_content.find("<![CDATA[") {
            if let Some(cdata_end) = tag_content.find("]]>") {
                let content = &tag_content[cdata_start + 9..cdata_end];
                return Some(content.trim().to_string());
            }
        }

        // No CDATA, return plain content
        Some(tag_content.trim().to_string())
    }

    /// Extract block content (entire tag with nested content)
    fn extract_block(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        let start = text.find(&open_tag)? + open_tag.len();
        let end = text[start..].find(&close_tag)? + start;

        Some(text[start..end].to_string())
    }

    /// Extract all ARPT tags in Affected_Airports block
    fn extract_all_airports(program_block: &str) -> Vec<String> {
        let mut airports = Vec::new();
        let mut search_start = 0;

        // Find the Affected_Airports block
        if let Some(airports_block) = Self::extract_block(program_block, "Affected_Airports") {
            while let Some(pos) = airports_block[search_start..].find("<ARPT>") {
                let arpt_start = search_start + pos + 6; // length of "<ARPT>"
                if let Some(arpt_end) = airports_block[arpt_start..].find("</ARPT>") {
                    let code = airports_block[arpt_start..arpt_start + arpt_end].trim().to_string();
                    airports.push(code);
                    search_start = arpt_start + arpt_end + 7; // length of "</ARPT>"
                } else {
                    break;
                }
            }
        }

        airports
    }
}

// =============================================================================
// FAA STATUS-SPECIFIC TYPES
// =============================================================================

/// Airport delay information
#[derive(Debug, Clone)]
pub struct AirportDelay {
    /// 3-letter IATA airport code
    pub airport_code: String,
    /// Full airport name (optional)
    pub airport_name: Option<String>,
    /// City (optional)
    pub city: Option<String>,
    /// State code (optional)
    pub state: Option<String>,
    /// Type of delay
    pub delay_type: DelayType,
    /// Severity level
    pub severity: DelaySeverity,
    /// Average delay in minutes (if applicable)
    pub avg_delay_minutes: Option<i32>,
    /// Reason for delay (optional)
    pub reason: Option<String>,
    /// Expected end time (optional)
    pub end_time: Option<String>,
    /// Last updated timestamp (optional)
    pub last_updated: Option<String>,
}

/// Type of airport delay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayType {
    /// Airport or runway closure
    Closure,
    /// Ground stop (no departures allowed)
    GroundStop,
    /// Ground delay program
    GroundDelay,
    /// Departure delays
    DepartureDelay,
    /// Arrival delays
    ArrivalDelay,
    /// Airspace flow program
    AirspaceFlowProgram,
}

impl DelayType {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Closure => "Closure",
            Self::GroundStop => "Ground Stop",
            Self::GroundDelay => "Ground Delay",
            Self::DepartureDelay => "Departure Delay",
            Self::ArrivalDelay => "Arrival Delay",
            Self::AirspaceFlowProgram => "Airspace Flow Program",
        }
    }
}

/// Severity level of delay
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DelaySeverity {
    /// Normal operations (minimal delay)
    Normal,
    /// Minor delays (< 15 min)
    Minor,
    /// Moderate delays (15-30 min)
    Moderate,
    /// Major delays (30-60 min)
    Major,
    /// Severe delays (> 60 min) or closures
    Severe,
}

impl DelaySeverity {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Minor => "Minor",
            Self::Moderate => "Moderate",
            Self::Major => "Major",
            Self::Severe => "Severe",
        }
    }
}

/// Airport status information
#[derive(Debug, Clone)]
pub struct AirportStatus {
    /// Last update timestamp
    pub timestamp: String,
    /// List of delays
    pub delays: Vec<AirportDelay>,
    /// Total number of delays
    pub count: usize,
}
