//! OpenWeatherMap connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    OpenWeatherMapParser, WeatherCurrent, WeatherForecast, AirPollution,
};

/// OpenWeatherMap connector
///
/// Provides access to weather data, forecasts, and air pollution information.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::open_weather_map::OpenWeatherMapConnector;
///
/// let connector = OpenWeatherMapConnector::from_env();
///
/// // Get current weather
/// let weather = connector.get_current_weather("London").await?;
///
/// // Get 5-day forecast
/// let forecast = connector.get_forecast("London").await?;
///
/// // Get air pollution
/// let pollution = connector.get_air_pollution(51.5085, -0.1257).await?;
/// ```
pub struct OpenWeatherMapConnector {
    client: Client,
    auth: OpenWeatherMapAuth,
    endpoints: OpenWeatherMapEndpoints,
    _testnet: bool,
}

impl OpenWeatherMapConnector {
    /// Create new OpenWeatherMap connector with authentication
    pub fn new(auth: OpenWeatherMapAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: OpenWeatherMapEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `OPENWEATHERMAP_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(OpenWeatherMapAuth::from_env())
    }

    /// Internal: Make GET request to OpenWeatherMap API
    async fn get(
        &self,
        endpoint: OpenWeatherMapEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication and units=metric
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

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

        // Check for OpenWeatherMap API errors
        OpenWeatherMapParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CORE API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get current weather by city name
    ///
    /// # Arguments
    /// - `city` - City name (e.g., "London", "New York", "Tokyo")
    ///
    /// # Returns
    /// Current weather data including temperature, humidity, wind, etc.
    pub async fn get_current_weather(&self, city: &str) -> ExchangeResult<WeatherCurrent> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), city.to_string());

        let response = self.get(OpenWeatherMapEndpoint::WeatherCity, params).await?;
        OpenWeatherMapParser::parse_current_weather(&response)
    }

    /// Get current weather by coordinates
    ///
    /// # Arguments
    /// - `lat` - Latitude
    /// - `lon` - Longitude
    ///
    /// # Returns
    /// Current weather data for the specified location
    pub async fn get_current_weather_coords(&self, lat: f64, lon: f64) -> ExchangeResult<WeatherCurrent> {
        let mut params = HashMap::new();
        params.insert("lat".to_string(), lat.to_string());
        params.insert("lon".to_string(), lon.to_string());

        let response = self.get(OpenWeatherMapEndpoint::WeatherCoords, params).await?;
        OpenWeatherMapParser::parse_current_weather(&response)
    }

    /// Get 5-day forecast by city name
    ///
    /// # Arguments
    /// - `city` - City name
    ///
    /// # Returns
    /// 5-day forecast with 3-hour intervals
    pub async fn get_forecast(&self, city: &str) -> ExchangeResult<WeatherForecast> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), city.to_string());

        let response = self.get(OpenWeatherMapEndpoint::ForecastCity, params).await?;
        OpenWeatherMapParser::parse_forecast(&response)
    }

    /// Get 5-day forecast by coordinates
    ///
    /// # Arguments
    /// - `lat` - Latitude
    /// - `lon` - Longitude
    ///
    /// # Returns
    /// 5-day forecast for the specified location
    pub async fn get_forecast_coords(&self, lat: f64, lon: f64) -> ExchangeResult<WeatherForecast> {
        let mut params = HashMap::new();
        params.insert("lat".to_string(), lat.to_string());
        params.insert("lon".to_string(), lon.to_string());

        let response = self.get(OpenWeatherMapEndpoint::ForecastCoords, params).await?;
        OpenWeatherMapParser::parse_forecast(&response)
    }

    /// Get current air pollution by coordinates
    ///
    /// # Arguments
    /// - `lat` - Latitude
    /// - `lon` - Longitude
    ///
    /// # Returns
    /// Air pollution data including AQI and pollutant concentrations
    pub async fn get_air_pollution(&self, lat: f64, lon: f64) -> ExchangeResult<AirPollution> {
        let mut params = HashMap::new();
        params.insert("lat".to_string(), lat.to_string());
        params.insert("lon".to_string(), lon.to_string());

        let response = self.get(OpenWeatherMapEndpoint::AirPollution, params).await?;
        OpenWeatherMapParser::parse_air_pollution(&response)
    }

    /// Get historical air pollution by coordinates
    ///
    /// # Arguments
    /// - `lat` - Latitude
    /// - `lon` - Longitude
    /// - `start` - Start time (Unix timestamp)
    /// - `end` - End time (Unix timestamp)
    ///
    /// # Returns
    /// Historical air pollution data for the specified time range
    pub async fn get_air_pollution_history(
        &self,
        lat: f64,
        lon: f64,
        start: u64,
        end: u64,
    ) -> ExchangeResult<AirPollution> {
        let mut params = HashMap::new();
        params.insert("lat".to_string(), lat.to_string());
        params.insert("lon".to_string(), lon.to_string());
        params.insert("start".to_string(), start.to_string());
        params.insert("end".to_string(), end.to_string());

        let response = self.get(OpenWeatherMapEndpoint::AirPollutionHistory, params).await?;
        OpenWeatherMapParser::parse_air_pollution(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get weather for major financial cities
    ///
    /// Returns weather data for: New York, London, Tokyo, Hong Kong, Shanghai, Singapore
    pub async fn get_weather_major_cities(&self) -> ExchangeResult<Vec<(String, WeatherCurrent)>> {
        let cities = vec![
            "New York",
            "London",
            "Tokyo",
            "Hong Kong",
            "Shanghai",
            "Singapore",
        ];

        let mut results = Vec::new();
        for city in cities {
            match self.get_current_weather(city).await {
                Ok(weather) => results.push((city.to_string(), weather)),
                Err(_) => continue, // Skip cities with errors
            }
        }

        Ok(results)
    }

    /// Get just the temperature for a city
    ///
    /// # Arguments
    /// - `city` - City name
    ///
    /// # Returns
    /// Temperature in Celsius
    pub async fn get_temperature(&self, city: &str) -> ExchangeResult<f64> {
        let weather = self.get_current_weather(city).await?;
        Ok(weather.main.temp)
    }

    /// Check for extreme weather conditions
    ///
    /// # Arguments
    /// - `city` - City name
    ///
    /// # Returns
    /// true if extreme conditions detected (temp < -10°C or > 40°C, wind > 20 m/s, humidity > 90%)
    pub async fn get_extreme_weather(&self, city: &str) -> ExchangeResult<bool> {
        let weather = self.get_current_weather(city).await?;

        let extreme = weather.main.temp < -10.0
            || weather.main.temp > 40.0
            || weather.wind.speed > 20.0
            || weather.main.humidity > 90;

        Ok(extreme)
    }

    /// Get weather for key commodity production regions
    ///
    /// # Arguments
    /// - `commodity` - Commodity name (wheat, coffee, oil, gold)
    ///
    /// # Returns
    /// Weather data for the primary production region
    pub async fn get_commodity_weather(&self, commodity: &str) -> ExchangeResult<WeatherCurrent> {
        let city = match commodity.to_lowercase().as_str() {
            "wheat" => "Kansas City",     // US wheat belt
            "coffee" => "Sao Paulo",      // Brazil coffee region
            "oil" => "Houston",           // US oil hub
            "gold" => "Johannesburg",     // South Africa gold region
            "corn" => "Des Moines",       // US corn belt
            "soybeans" => "Chicago",      // US soybean region
            "cotton" => "Memphis",        // US cotton belt
            "cocoa" => "Accra",          // Ghana cocoa region
            "sugar" => "Sao Paulo",      // Brazil sugar region
            "rice" => "Bangkok",         // Thailand rice region
            _ => return Err(ExchangeError::Parse(format!("Unknown commodity: {}", commodity))),
        };

        self.get_current_weather(city).await
    }
}
