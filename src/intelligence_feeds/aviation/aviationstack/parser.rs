//! AviationStack response parsers
//!
//! Parse JSON responses to domain types based on AviationStack API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct AviationStackParser;

// ═══════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Flight data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvFlight {
    /// Flight date (e.g., "2024-01-15")
    pub flight_date: String,
    /// Flight status (e.g., "active", "scheduled", "landed")
    pub flight_status: String,
    /// Departure airport information
    pub departure: AvAirport,
    /// Arrival airport information
    pub arrival: AvAirport,
    /// Airline information
    pub airline: AvAirline,
    /// Flight information
    pub flight: AvFlightInfo,
}

/// Airport information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvAirport {
    /// Airport name
    pub airport: Option<String>,
    /// Timezone (e.g., "America/New_York")
    pub timezone: Option<String>,
    /// IATA code (e.g., "JFK")
    pub iata: Option<String>,
    /// ICAO code (e.g., "KJFK")
    pub icao: Option<String>,
    /// Terminal
    pub terminal: Option<String>,
    /// Gate
    pub gate: Option<String>,
    /// Scheduled time (ISO 8601)
    pub scheduled: Option<String>,
    /// Estimated time (ISO 8601)
    pub estimated: Option<String>,
    /// Actual time (ISO 8601)
    pub actual: Option<String>,
}

/// Airline information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvAirline {
    /// Airline name
    pub name: Option<String>,
    /// IATA code (e.g., "AA")
    pub iata: Option<String>,
    /// ICAO code (e.g., "AAL")
    pub icao: Option<String>,
}

/// Flight information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvFlightInfo {
    /// Flight number (e.g., "100")
    pub number: Option<String>,
    /// IATA code (e.g., "AA100")
    pub iata: Option<String>,
    /// ICAO code (e.g., "AAL100")
    pub icao: Option<String>,
}

/// Flight route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvRoute {
    /// Departure IATA code
    pub departure_iata: Option<String>,
    /// Arrival IATA code
    pub arrival_iata: Option<String>,
    /// Airline IATA code
    pub airline_iata: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════
// PARSER IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════

impl AviationStackParser {
    /// Check for API error in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let code = error.get("code")
                .and_then(|v| v.as_i64())
                .unwrap_or(-1) as i32;
            let message = error.get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api { code, message });
        }
        Ok(())
    }

    /// Parse flights response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "pagination": {...},
    ///   "data": [...]
    /// }
    /// ```
    pub fn parse_flights(response: &Value) -> ExchangeResult<Vec<AvFlight>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(Self::parse_flight)
            .collect()
    }

    /// Parse single flight object
    fn parse_flight(flight: &Value) -> ExchangeResult<AvFlight> {
        Ok(AvFlight {
            flight_date: Self::get_str(flight, "flight_date")
                .unwrap_or("")
                .to_string(),
            flight_status: Self::get_str(flight, "flight_status")
                .unwrap_or("")
                .to_string(),
            departure: Self::parse_airport(flight.get("departure"))?,
            arrival: Self::parse_airport(flight.get("arrival"))?,
            airline: Self::parse_airline(flight.get("airline"))?,
            flight: Self::parse_flight_info(flight.get("flight"))?,
        })
    }

    /// Parse airport object
    fn parse_airport(airport: Option<&Value>) -> ExchangeResult<AvAirport> {
        let airport = airport.ok_or_else(|| {
            ExchangeError::Parse("Missing airport object".to_string())
        })?;

        Ok(AvAirport {
            airport: Self::get_str(airport, "airport").map(|s| s.to_string()),
            timezone: Self::get_str(airport, "timezone").map(|s| s.to_string()),
            iata: Self::get_str(airport, "iata").map(|s| s.to_string()),
            icao: Self::get_str(airport, "icao").map(|s| s.to_string()),
            terminal: Self::get_str(airport, "terminal").map(|s| s.to_string()),
            gate: Self::get_str(airport, "gate").map(|s| s.to_string()),
            scheduled: Self::get_str(airport, "scheduled").map(|s| s.to_string()),
            estimated: Self::get_str(airport, "estimated").map(|s| s.to_string()),
            actual: Self::get_str(airport, "actual").map(|s| s.to_string()),
        })
    }

    /// Parse airline object
    fn parse_airline(airline: Option<&Value>) -> ExchangeResult<AvAirline> {
        let airline = airline.ok_or_else(|| {
            ExchangeError::Parse("Missing airline object".to_string())
        })?;

        Ok(AvAirline {
            name: Self::get_str(airline, "name").map(|s| s.to_string()),
            iata: Self::get_str(airline, "iata").map(|s| s.to_string()),
            icao: Self::get_str(airline, "icao").map(|s| s.to_string()),
        })
    }

    /// Parse flight info object
    fn parse_flight_info(flight: Option<&Value>) -> ExchangeResult<AvFlightInfo> {
        let flight = flight.ok_or_else(|| {
            ExchangeError::Parse("Missing flight object".to_string())
        })?;

        Ok(AvFlightInfo {
            number: Self::get_str(flight, "number").map(|s| s.to_string()),
            iata: Self::get_str(flight, "iata").map(|s| s.to_string()),
            icao: Self::get_str(flight, "icao").map(|s| s.to_string()),
        })
    }

    /// Parse routes response
    pub fn parse_routes(response: &Value) -> ExchangeResult<Vec<AvRoute>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(Self::parse_route)
            .collect()
    }

    /// Parse single route object
    fn parse_route(route: &Value) -> ExchangeResult<AvRoute> {
        Ok(AvRoute {
            departure_iata: Self::get_str(route, "departure_iata").map(|s| s.to_string()),
            arrival_iata: Self::get_str(route, "arrival_iata").map(|s| s.to_string()),
            airline_iata: Self::get_str(route, "airline_iata").map(|s| s.to_string()),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn get_str<'a>(obj: &'a Value, key: &str) -> Option<&'a str> {
        obj.get(key).and_then(|v| v.as_str())
    }
}
