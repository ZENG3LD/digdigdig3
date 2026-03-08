//! OpenWeatherMap API endpoints

/// Base URLs for OpenWeatherMap API
pub struct OpenWeatherMapEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for OpenWeatherMapEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.openweathermap.org/data/2.5",
            ws_base: None, // OpenWeatherMap does not support WebSocket
        }
    }
}

/// OpenWeatherMap API endpoint enum
#[derive(Debug, Clone)]
pub enum OpenWeatherMapEndpoint {
    /// Get current weather by city name
    WeatherCity,
    /// Get current weather by coordinates
    WeatherCoords,
    /// Get 5-day forecast by city name
    ForecastCity,
    /// Get 5-day forecast by coordinates
    ForecastCoords,
    /// Get current air pollution by coordinates
    AirPollution,
    /// Get historical air pollution by coordinates
    AirPollutionHistory,
}

impl OpenWeatherMapEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::WeatherCity => "/weather",
            Self::WeatherCoords => "/weather",
            Self::ForecastCity => "/forecast",
            Self::ForecastCoords => "/forecast",
            Self::AirPollution => "/air_pollution",
            Self::AirPollutionHistory => "/air_pollution/history",
        }
    }
}
