//! Space-Track.org API endpoints
//!
//! Space-Track uses a REST-like query API where predicates are embedded in URL segments:
//! `/basicspacedata/query/class/{class}/{predicates}/format/json`
//!
//! This module provides both a high-level `SpaceTrackEndpoint` enum for common queries
//! and a flexible `SpaceTrackQuery` builder for constructing arbitrary queries.

// ═══════════════════════════════════════════════════════════════════════════════
// BASE URLS
// ═══════════════════════════════════════════════════════════════════════════════

/// Base URLs for Space-Track API
pub struct SpaceTrackEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for SpaceTrackEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.space-track.org",
            ws_base: None, // Space-Track does not support WebSocket
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SPACE-TRACK DATA CLASSES
// ═══════════════════════════════════════════════════════════════════════════════

/// Space-Track data classes (corresponds to `class/` in the query URL)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceTrackClass {
    /// Satellite catalog (SATCAT) — all known space objects
    SatCat,
    /// General Perturbations (GP/TLE orbital elements)
    Gp,
    /// General Perturbations history
    GpHistory,
    /// Decay predictions for deorbiting objects
    Decay,
    /// Launch site information
    LaunchSite,
    /// Tracking & Impact Predictions (TIP)
    Tip,
    /// Conjunction data messages (CDM) — collision warnings
    Cdm,
    /// Box score: conjunction statistics
    BoxScore,
    /// Organization information
    Organization,
    /// Custom class string (for classes not listed above)
    Custom(String),
}

impl SpaceTrackClass {
    /// Get class name for URL construction
    pub fn as_str(&self) -> &str {
        match self {
            Self::SatCat => "satcat",
            Self::Gp => "gp",
            Self::GpHistory => "gp_history",
            Self::Decay => "decay",
            Self::LaunchSite => "launch_site",
            Self::Tip => "tip",
            Self::Cdm => "cdm_public",
            Self::BoxScore => "boxscore",
            Self::Organization => "organization",
            Self::Custom(s) => s.as_str(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FLEXIBLE QUERY BUILDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Builder for flexible Space-Track API queries
///
/// Space-Track URLs follow the pattern:
/// `/basicspacedata/query/class/{class}/{predicate1}/{value1}/{predicate2}/{value2}/format/json`
///
/// # Examples
///
/// ```ignore
/// // Get TLE for ISS (NORAD ID 25544)
/// let query = SpaceTrackQuery::new(SpaceTrackClass::Gp)
///     .filter("NORAD_CAT_ID", "25544")
///     .format_json();
///
/// // Get 50 most recent launches ordered by launch date
/// let query = SpaceTrackQuery::new(SpaceTrackClass::SatCat)
///     .order_by("LAUNCH", SortOrder::Desc)
///     .limit(50)
///     .format_json();
///
/// // Get debris launched after 2020
/// let query = SpaceTrackQuery::new(SpaceTrackClass::Gp)
///     .filter("OBJECT_TYPE", "DEBRIS")
///     .filter_range("EPOCH", "2020-01-01", "")
///     .order_by("LAUNCH", SortOrder::Desc)
///     .limit(100)
///     .format_json();
/// ```
#[derive(Debug, Clone, Default)]
pub struct SpaceTrackQuery {
    class: String,
    predicates: Vec<(String, String)>,
    limit: Option<u32>,
    order_by: Option<(String, SortOrder)>,
    distinct: bool,
}

/// Sort order for Space-Track queries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

impl SpaceTrackQuery {
    /// Create a new query for the given class
    pub fn new(class: SpaceTrackClass) -> Self {
        Self {
            class: class.as_str().to_string(),
            predicates: Vec::new(),
            limit: None,
            order_by: None,
            distinct: false,
        }
    }

    /// Add an equality filter: `/field/value`
    pub fn filter(mut self, field: &str, value: &str) -> Self {
        self.predicates.push((field.to_string(), value.to_string()));
        self
    }

    /// Add a range filter using `>`, `<`, `--` syntax: `/field/>start<end`
    ///
    /// - `start` only: `>start`
    /// - `end` only: `<end`
    /// - Both: `>start<end`
    pub fn filter_range(mut self, field: &str, start: &str, end: &str) -> Self {
        let value = match (start.is_empty(), end.is_empty()) {
            (false, false) => format!(">{}<{}", start, end),
            (false, true) => format!(">{}", start),
            (true, false) => format!("<{}", end),
            (true, true) => return self, // No-op if both empty
        };
        self.predicates.push((field.to_string(), value));
        self
    }

    /// Add a "not equal" filter: `/field/^^value`
    pub fn filter_not(mut self, field: &str, value: &str) -> Self {
        self.predicates.push((field.to_string(), format!("^^{}", value)));
        self
    }

    /// Add a "null check" filter: `/field/null-val` or `/field/not-null-val`
    pub fn filter_null(mut self, field: &str, is_null: bool) -> Self {
        let value = if is_null { "null-val" } else { "not-null-val" };
        self.predicates.push((field.to_string(), value.to_string()));
        self
    }

    /// Set ORDER BY: `/orderby/{field} {asc|desc}`
    pub fn order_by(mut self, field: &str, order: SortOrder) -> Self {
        self.order_by = Some((field.to_string(), order));
        self
    }

    /// Set result LIMIT: `/limit/{n}`
    pub fn limit(mut self, n: u32) -> Self {
        self.limit = Some(n);
        self
    }

    /// Set DISTINCT flag: `/distinct/true`
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Build the URL path string for this query (always uses JSON format)
    pub fn build_path(&self) -> String {
        let mut parts = vec![
            String::from("/basicspacedata/query/class"),
            self.class.clone(),
        ];

        for (field, value) in &self.predicates {
            parts.push(field.clone());
            parts.push(value.clone());
        }

        if let Some((field, order)) = &self.order_by {
            parts.push("orderby".to_string());
            parts.push(format!("{} {}", field, order.as_str()));
        }

        if let Some(n) = self.limit {
            parts.push("limit".to_string());
            parts.push(n.to_string());
        }

        if self.distinct {
            parts.push("distinct".to_string());
            parts.push("true".to_string());
        }

        parts.push("format".to_string());
        parts.push("json".to_string());

        parts.join("/")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HIGH-LEVEL ENDPOINT ENUM (pre-built common queries)
// ═══════════════════════════════════════════════════════════════════════════════

/// Space-Track API endpoint enum
///
/// Provides pre-built paths for the most common queries.
/// For custom queries with dynamic predicates, use `SpaceTrackQuery` directly.
#[derive(Debug, Clone)]
pub enum SpaceTrackEndpoint {
    // Authentication endpoint
    Login,

    // Pre-built common queries (built via SpaceTrackQuery)
    /// Recent satellite launches (configurable limit, default 25)
    SatelliteCatalog { limit: u32 },

    /// TLE/GP data for a specific satellite by NORAD ID
    GeneralPerturbations { norad_id: u32 },

    /// Recent decay predictions (configurable limit, default 25)
    Decay { limit: u32 },

    /// Space debris objects (configurable limit, default 50)
    Debris { limit: u32 },

    /// All launch sites
    LaunchSites,

    /// Tracking & Impact Predictions (configurable limit, default 25)
    Tip { limit: u32 },

    /// Arbitrary query path built by `SpaceTrackQuery`
    Custom(SpaceTrackQuery),

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════
    /// Conjunction Data Messages (CDM) — collision warning data (public subset)
    CdmPublic { limit: u32 },
    /// GP History — historical orbital elements for a satellite
    GpHistory { norad_id: u32 },
    /// Boxscore — aggregate conjunction statistics by country/owner
    Boxscore,
    /// SATCAT Change log — satellites whose catalog entries have changed
    SatcatChange { days_back: u32 },
    /// OMM (Orbital Mean-Elements Message) — JSON OMM format for a satellite
    Omm { norad_id: u32 },
}

impl SpaceTrackEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            Self::Login => "/ajaxauth/login".to_string(),

            Self::SatelliteCatalog { limit } => SpaceTrackQuery::new(SpaceTrackClass::SatCat)
                .order_by("LAUNCH", SortOrder::Desc)
                .limit(*limit)
                .build_path(),

            Self::GeneralPerturbations { norad_id } => SpaceTrackQuery::new(SpaceTrackClass::Gp)
                .filter("NORAD_CAT_ID", &norad_id.to_string())
                .build_path(),

            Self::Decay { limit } => SpaceTrackQuery::new(SpaceTrackClass::Decay)
                .order_by("DECAY_EPOCH", SortOrder::Desc)
                .limit(*limit)
                .build_path(),

            Self::Debris { limit } => SpaceTrackQuery::new(SpaceTrackClass::Gp)
                .filter("OBJECT_TYPE", "DEBRIS")
                .order_by("LAUNCH", SortOrder::Desc)
                .limit(*limit)
                .build_path(),

            Self::LaunchSites => {
                SpaceTrackQuery::new(SpaceTrackClass::LaunchSite).build_path()
            }

            Self::Tip { limit } => SpaceTrackQuery::new(SpaceTrackClass::Tip)
                .limit(*limit)
                .build_path(),

            Self::Custom(query) => query.build_path(),

            // C7 additions
            Self::CdmPublic { limit } => SpaceTrackQuery::new(SpaceTrackClass::Cdm)
                .order_by("TCA", SortOrder::Desc)
                .limit(*limit)
                .build_path(),

            Self::GpHistory { norad_id } => SpaceTrackQuery::new(SpaceTrackClass::GpHistory)
                .filter("NORAD_CAT_ID", &norad_id.to_string())
                .order_by("EPOCH", SortOrder::Desc)
                .build_path(),

            Self::Boxscore => SpaceTrackQuery::new(SpaceTrackClass::BoxScore)
                .build_path(),

            Self::SatcatChange { days_back } => {
                let since = format!("now-{}d", days_back);
                SpaceTrackQuery::new(SpaceTrackClass::SatCat)
                    .filter_range("CURRENT", &since, "")
                    .order_by("LAUNCH", SortOrder::Desc)
                    .build_path()
            }

            Self::Omm { norad_id } => {
                // OMM uses a different format — direct JSON query
                format!(
                    "/basicspacedata/query/class/gp/NORAD_CAT_ID/{}/format/json/orderby/EPOCH%20desc/limit/1",
                    norad_id
                )
            }
        }
    }
}

impl Default for SpaceTrackEndpoint {
    fn default() -> Self {
        Self::SatelliteCatalog { limit: 25 }
    }
}
