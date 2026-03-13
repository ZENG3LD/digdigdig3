//! # Raydium Endpoints
//!
//! API URLs and endpoint enum for Raydium DEX.
//!
//! ## Important Notes
//!
//! - Raydium is an AMM-based DEX on Solana, not a traditional CEX
//! - All APIs are public (no authentication required)
//! - Token identification uses Solana mint addresses, not symbols
//! - Pool IDs are Solana public keys (Base58-encoded)

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URLs for Raydium API
#[derive(Debug, Clone)]
pub struct RaydiumUrls {
    pub api_v3: &'static str,
    pub trade_api: &'static str,
}

impl RaydiumUrls {
    /// Mainnet URLs (production)
    pub const MAINNET: Self = Self {
        api_v3: "https://api-v3.raydium.io",
        trade_api: "https://transaction-v1.raydium.io",
    };

    /// Devnet URLs (testing)
    pub const DEVNET: Self = Self {
        api_v3: "https://api-v3-devnet.raydium.io",
        trade_api: "https://transaction-v1.raydium.io", // Same for devnet
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Raydium API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RaydiumEndpoint {
    // === MAIN - Platform Info ===
    Version,
    Rpcs,
    AutoFee,

    // === MINT - Token Information ===
    MintList,
    MintIds,
    MintPrice,

    // === POOLS - Liquidity Pool Data ===
    PoolList,
    PoolIds,
    PoolByMint,
    PoolPositions,

    // === FARMS - Yield Farming ===
    FarmList,
    FarmIds,

    // === IDO - Initial DEX Offering ===
    IdoPoolKeys,

    // === TRADE API - Quote & Swap Serialization ===
    SwapQuoteBaseIn,
    SwapQuoteBaseOut,
    SwapTransactionBaseIn,
    SwapTransactionBaseOut,

    // === MAIN (Extended) ===
    /// GET /main/chain-time — current Solana cluster time
    ChainTime,
    /// GET /main/info — platform summary (TVL, 24h volume, fee revenue)
    PlatformInfo,

    // === POOLS (Extended) ===
    /// GET /pools/line/price — pool token price history (OHLCV line data)
    PoolPriceHistory,
    /// GET /pools/line/liquidity — pool liquidity history over time
    PoolLiquidityHistory,
    /// GET /pools/info/stats — aggregate TVL and volume stats across all pools
    PoolStats,

    // === POOL CONFIGS ===
    /// GET /clmm/configs — CLMM (concentrated liquidity) pool configuration tiers
    ClmmConfigs,
    /// GET /cpmm/configs — CPMM (constant product) pool configuration tiers
    CpmmConfigs,

    // === FARMS (Extended) ===
    /// GET /farms/info/mine — farms owned/staked by a wallet address
    FarmOwnership,
}

impl RaydiumEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Main
            Self::Version => "/main/version",
            Self::Rpcs => "/main/rpcs",
            Self::AutoFee => "/main/auto-fee",

            // Mint
            Self::MintList => "/mint/list",
            Self::MintIds => "/mint/ids",
            Self::MintPrice => "/mint/price",

            // Pools
            Self::PoolList => "/pools/info/list",
            Self::PoolIds => "/pools/info/ids",
            Self::PoolByMint => "/pools/info/mint",
            Self::PoolPositions => "/pools/position/list",

            // Farms
            Self::FarmList => "/farms/info/list",
            Self::FarmIds => "/farms/info/ids",

            // IDO
            Self::IdoPoolKeys => "/ido/pool-keys",

            // Trade API
            Self::SwapQuoteBaseIn => "/compute/swap-base-in",
            Self::SwapQuoteBaseOut => "/compute/swap-base-out",
            Self::SwapTransactionBaseIn => "/transaction/swap-base-in",
            Self::SwapTransactionBaseOut => "/transaction/swap-base-out",

            // Main (Extended)
            Self::ChainTime => "/main/chain-time",
            Self::PlatformInfo => "/main/info",

            // Pools (Extended)
            Self::PoolPriceHistory => "/pools/line/price",
            Self::PoolLiquidityHistory => "/pools/line/liquidity",
            Self::PoolStats => "/pools/info/stats",

            // Pool Configs
            Self::ClmmConfigs => "/clmm/configs",
            Self::CpmmConfigs => "/cpmm/configs",

            // Farms (Extended)
            Self::FarmOwnership => "/farms/info/mine",
        }
    }

    /// Get full URL for endpoint
    pub fn url(&self, urls: &RaydiumUrls) -> String {
        let base = match self {
            Self::SwapQuoteBaseIn
            | Self::SwapQuoteBaseOut
            | Self::SwapTransactionBaseIn
            | Self::SwapTransactionBaseOut => urls.trade_api,
            _ => urls.api_v3,
        };

        format!("{}{}", base, self.path())
    }

    /// HTTP method for this endpoint
    pub fn method(&self) -> &'static str {
        match self {
            Self::SwapTransactionBaseIn | Self::SwapTransactionBaseOut => "POST",
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL/MINT FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Common Solana token mint addresses
pub mod well_known_mints {
    /// Wrapped SOL (native token)
    pub const SOL: &str = "So11111111111111111111111111111111111111112";

    /// USD Coin (Circle)
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    /// Tether USD
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";

    /// Raydium Token
    pub const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";

    /// Serum Token
    pub const SRM: &str = "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt";

    /// RAY-SOL pool ID (legacy, kept for reference)
    pub const RAY_SOL_POOL: &str = "AVs9TA4nWDzfPJE9gGVNJMVhcQy3V9PGazuz33BfG2RA";

    /// SOL-USDC AMM V4 pool ID (canonical Raydium pool)
    pub const SOL_USDC_POOL: &str = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2";

    /// Raydium AMM V4 program ID
    pub const AMM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
}

/// Validate Solana mint/pool address format
///
/// Basic validation - checks Base58 character set and approximate length.
/// Full validation requires Base58 decoding to 32 bytes.
pub fn validate_solana_address(address: &str) -> bool {
    // Solana addresses are 32-44 characters (Base58 encoding)
    if address.len() < 32 || address.len() > 44 {
        return false;
    }

    // Check Base58 alphabet: [1-9A-HJ-NP-Za-km-z]
    // (excludes 0, O, I, l to avoid confusion)
    address
        .chars()
        .all(|c| c.is_ascii_alphanumeric() && c != '0' && c != 'O' && c != 'I' && c != 'l')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(RaydiumEndpoint::MintList.path(), "/mint/list");
        assert_eq!(RaydiumEndpoint::PoolList.path(), "/pools/info/list");
        assert_eq!(RaydiumEndpoint::SwapQuoteBaseIn.path(), "/compute/swap-base-in");
    }

    #[test]
    fn test_endpoint_urls() {
        let urls = RaydiumUrls::MAINNET;

        assert_eq!(
            RaydiumEndpoint::MintList.url(&urls),
            "https://api-v3.raydium.io/mint/list"
        );

        assert_eq!(
            RaydiumEndpoint::SwapQuoteBaseIn.url(&urls),
            "https://transaction-v1.raydium.io/compute/swap-base-in"
        );
    }

    #[test]
    fn test_validate_solana_address() {
        // Valid addresses
        assert!(validate_solana_address(well_known_mints::SOL));
        assert!(validate_solana_address(well_known_mints::USDC));
        assert!(validate_solana_address(well_known_mints::SOL_USDC_POOL));

        // Invalid addresses
        assert!(!validate_solana_address(""));
        assert!(!validate_solana_address("short"));
        assert!(!validate_solana_address("contains_invalid_0_character_123456789"));
        assert!(!validate_solana_address("toooooooooooooooooooooooooooooooooooooooooooooolong"));
    }
}
