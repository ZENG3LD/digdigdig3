//! BEA API endpoints

/// Base URLs for BEA API
pub struct BeaEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for BeaEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://apps.bea.gov/api/data",
            ws_base: None, // BEA does not support WebSocket
        }
    }
}

/// BEA API endpoint enum
///
/// All BEA requests go to the same base URL with different `method` parameter
#[derive(Debug, Clone)]
pub enum BeaEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // METADATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get list of available datasets (method=GETDATASETLIST)
    GetDatasetList,

    /// Get parameters for a dataset (method=GetParameterList)
    GetParameterList,

    /// Get values for a parameter (method=GetParameterValues)
    GetParameterValues,

    /// Get filtered parameter values (method=GetParameterValuesFiltered)
    GetParameterValuesFiltered,

    // ═══════════════════════════════════════════════════════════════════════
    // DATA ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════
    /// Get data from a dataset (method=GetData)
    GetData,
}

impl BeaEndpoint {
    /// Get the method parameter value
    pub fn method(&self) -> &'static str {
        match self {
            Self::GetDatasetList => "GETDATASETLIST",
            Self::GetParameterList => "GetParameterList",
            Self::GetParameterValues => "GetParameterValues",
            Self::GetParameterValuesFiltered => "GetParameterValuesFiltered",
            Self::GetData => "GetData",
        }
    }
}

/// BEA dataset names
///
/// Key datasets available through the BEA API
#[derive(Debug, Clone, Copy)]
pub enum BeaDataset {
    /// National Income and Product Accounts (GDP!)
    NIPA,
    /// NIPA underlying detail
    NIUnderlyingDetail,
    /// Multinational Enterprises
    MNE,
    /// Fixed Assets
    FixedAssets,
    /// International Transactions
    ITA,
    /// International Investment Position
    IIP,
    /// GDP by Industry
    GDPbyIndustry,
    /// Regional Economic Accounts
    Regional,
    /// Underlying GDP detail by industry
    UnderlyingGDPbyIndustry,
    /// API metadata
    APIDatasetMetaData,
}

impl BeaDataset {
    /// Get dataset name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NIPA => "NIPA",
            Self::NIUnderlyingDetail => "NIUnderlyingDetail",
            Self::MNE => "MNE",
            Self::FixedAssets => "FixedAssets",
            Self::ITA => "ITA",
            Self::IIP => "IIP",
            Self::GDPbyIndustry => "GDPbyIndustry",
            Self::Regional => "Regional",
            Self::UnderlyingGDPbyIndustry => "UnderlyingGDPbyIndustry",
            Self::APIDatasetMetaData => "APIDatasetMetaData",
        }
    }
}

/// NIPA table names
///
/// Key tables in the National Income and Product Accounts
#[derive(Debug, Clone, Copy)]
pub enum NipaTable {
    /// GDP and components
    T10101,
    /// GDP price index
    T10106,
    /// Personal income
    T20100,
    /// Government receipts and expenditures
    T30100,
    /// Foreign transactions
    T40100,
}

impl NipaTable {
    /// Get table name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::T10101 => "T10101",
            Self::T10106 => "T10106",
            Self::T20100 => "T20100",
            Self::T30100 => "T30100",
            Self::T40100 => "T40100",
        }
    }
}

/// Format dataset name for BEA API
///
/// BEA uses dataset names like "NIPA", "GDPbyIndustry"
/// This is different from crypto exchanges - there's no base/quote concept.
/// Dataset names are unique identifiers in the BEA API.
///
/// For compatibility with the Symbol type, we'll use:
/// - base = dataset_name
/// - quote = "" (empty)
pub fn format_dataset_name(symbol: &crate::core::types::Symbol) -> String {
    // For BEA, the "base" field contains the dataset name
    symbol.base.to_string()
}

/// Parse dataset name from BEA response to domain Symbol
///
/// BEA dataset names become the "base" field, with empty "quote"
pub fn parse_dataset_name(dataset_name: &str) -> crate::core::types::Symbol {
    crate::core::types::Symbol::new(dataset_name, "")
}
