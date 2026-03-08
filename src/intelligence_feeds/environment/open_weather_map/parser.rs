//! OpenWeatherMap response parsers
//!
//! Parse JSON responses to domain types based on OpenWeatherMap API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct OpenWeatherMapParser;

impl OpenWeatherMapParser {
    // ═══════════════════════════════════════════════════════════════════════
    // CURRENT WEATHER PARSER
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse current weather response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "coord": { "lon": -0.1257, "lat": 51.5085 },
    ///   "weather": [
    ///     {
    ///       "id": 300,
    ///       "main": "Drizzle",
    ///       "description": "light intensity drizzle",
    ///       "icon": "09d"
    ///     }
    ///   ],
    ///   "main": {
    ///     "temp": 280.32,
    ///     "feels_like": 278.99,
    ///     "temp_min": 279.15,
    ///     "temp_max": 281.15,
    ///     "pressure": 1012,
    ///     "humidity": 81
    ///   },
    ///   "wind": {
    ///     "speed": 4.1,
    ///     "deg": 80
    ///   },
    ///   "dt": 1485789600,
    ///   "sys": {
    ///     "country": "GB",
    ///     "sunrise": 1485762037,
    ///     "sunset": 1485794875
    ///   },
    ///   "name": "London"
    /// }
    /// ```
    pub fn parse_current_weather(response: &Value) -> ExchangeResult<WeatherCurrent> {
        let current: WeatherCurrent = serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse current weather: {}", e)))?;
        Ok(current)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FORECAST PARSER
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse 5-day forecast response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "list": [
    ///     {
    ///       "dt": 1485799200,
    ///       "main": { "temp": 280.32, ... },
    ///       "weather": [...],
    ///       "wind": { ... },
    ///       "dt_txt": "2017-01-30 18:00:00"
    ///     }
    ///   ],
    ///   "city": {
    ///     "name": "London",
    ///     "coord": { "lat": 51.5085, "lon": -0.1257 },
    ///     "country": "GB"
    ///   }
    /// }
    /// ```
    pub fn parse_forecast(response: &Value) -> ExchangeResult<WeatherForecast> {
        let forecast: WeatherForecast = serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse forecast: {}", e)))?;
        Ok(forecast)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // AIR POLLUTION PARSER
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse air pollution response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "coord": { "lon": -0.1257, "lat": 51.5085 },
    ///   "list": [
    ///     {
    ///       "dt": 1605182400,
    ///       "main": { "aqi": 1 },
    ///       "components": {
    ///         "co": 201.94,
    ///         "no": 0.01,
    ///         "no2": 0.5,
    ///         "o3": 68.66,
    ///         "so2": 0.64,
    ///         "pm2_5": 0.5,
    ///         "pm10": 0.54,
    ///         "nh3": 0.12
    ///       }
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_air_pollution(response: &Value) -> ExchangeResult<AirPollution> {
        let pollution: AirPollution = serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse air pollution: {}", e)))?;
        Ok(pollution)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(cod) = response.get("cod") {
            // cod can be a string or number
            let code_str = if let Some(s) = cod.as_str() {
                s
            } else if let Some(n) = cod.as_i64() {
                if n != 200 {
                    return Err(ExchangeError::Api {
                        code: n as i32,
                        message: response
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string(),
                    });
                }
                return Ok(());
            } else {
                return Ok(());
            };

            // String codes like "404", "401", etc.
            if code_str != "200" {
                let code = code_str.parse::<i32>().unwrap_or(0);
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                return Err(ExchangeError::Api { code, message });
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OPENWEATHERMAP-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Current weather data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherCurrent {
    pub coord: Coord,
    pub weather: Vec<WeatherCondition>,
    pub main: MainWeather,
    pub wind: Wind,
    pub visibility: Option<u32>,
    pub dt: u64,
    pub sys: WeatherSys,
    pub name: String,
}

/// Coordinates
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

/// Weather condition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherCondition {
    pub id: u32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

/// Main weather metrics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MainWeather {
    pub temp: f64,
    pub feels_like: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub pressure: u32,
    pub humidity: u32,
}

/// Wind data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Wind {
    pub speed: f64,
    pub deg: u32,
    pub gust: Option<f64>,
}

/// System data (country, sunrise, sunset)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherSys {
    pub country: String,
    pub sunrise: u64,
    pub sunset: u64,
}

/// 5-day forecast
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WeatherForecast {
    pub list: Vec<ForecastEntry>,
    pub city: ForecastCity,
}

/// Single forecast entry (3-hour interval)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForecastEntry {
    pub dt: u64,
    pub main: MainWeather,
    pub weather: Vec<WeatherCondition>,
    pub wind: Wind,
    pub dt_txt: String,
}

/// City information in forecast
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForecastCity {
    pub name: String,
    pub coord: Coord,
    pub country: String,
}

/// Air pollution data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AirPollution {
    pub coord: Coord,
    pub list: Vec<AirPollutionEntry>,
}

/// Single air pollution entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AirPollutionEntry {
    pub dt: u64,
    pub main: AirQualityIndex,
    pub components: PollutionComponents,
}

/// Air Quality Index
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AirQualityIndex {
    /// Air Quality Index: 1=Good, 2=Fair, 3=Moderate, 4=Poor, 5=Very Poor
    pub aqi: u32,
}

/// Pollution components (µg/m³)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PollutionComponents {
    /// Carbon monoxide
    pub co: f64,
    /// Nitrogen monoxide
    pub no: f64,
    /// Nitrogen dioxide
    pub no2: f64,
    /// Ozone
    pub o3: f64,
    /// Sulphur dioxide
    pub so2: f64,
    /// Fine particles matter (PM2.5)
    pub pm2_5: f64,
    /// Coarse particulate matter (PM10)
    pub pm10: f64,
    /// Ammonia
    pub nh3: f64,
}
