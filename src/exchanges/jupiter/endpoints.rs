//! # Jupiter Endpoints
//!
//! URL'ы и endpoint enum для Jupiter API.
//!
//! Jupiter использует Solana mint addresses вместо символов (например, SOL, USDC).

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Jupiter API
#[derive(Debug, Clone)]
pub struct JupiterUrls {
    /// Swap API v1 (все endpoints требуют API key с Oct 2025)
    pub swap_rest: &'static str,
    /// Metis Swap API (legacy, deprecated)
    pub metis_rest: &'static str,
    /// Price API V3 (требует API key)
    pub price_rest: &'static str,
    /// Tokens API V2 (требует API key)
    pub tokens_rest: &'static str,
}

impl JupiterUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        swap_rest: "https://api.jup.ag/swap/v1",
        metis_rest: "https://api.jup.ag/swap/v1",
        price_rest: "https://api.jup.ag/price/v3",
        tokens_rest: "https://api.jup.ag/tokens/v2",
    };

    /// Get base URL for primary API (price/tokens)
    pub fn base_url(&self) -> &str {
        "https://api.jup.ag"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Jupiter API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JupiterEndpoint {
    // === SWAP API ===
    /// GET /quote - request swap quote
    Quote,
    /// POST /swap - build swap transaction
    Swap,
    /// POST /swap-instructions - get swap instructions
    SwapInstructions,

    // === PRICE API ===
    /// GET /price/v3 - get token prices (max 50)
    Price,

    // === TOKENS API ===
    /// GET /tokens/v2/search - search tokens
    TokenSearch,
    /// GET /tokens/v2/tag - query by tag
    TokenTag,
    /// GET /tokens/v2/{category}/{interval} - top tokens
    TokenCategory,
    /// GET /tokens/v2/recent - recently created tokens
    TokenRecent,
}

impl JupiterEndpoint {
    /// Получить полный URL для endpoint'а
    pub fn url(&self, urls: &JupiterUrls) -> String {
        match self {
            Self::Quote => format!("{}/quote", urls.swap_rest),
            Self::Swap => format!("{}/swap", urls.swap_rest),
            Self::SwapInstructions => format!("{}/swap-instructions", urls.swap_rest),
            Self::Price => urls.price_rest.to_string(),
            Self::TokenSearch => format!("{}/search", urls.tokens_rest),
            Self::TokenTag => format!("{}/tag", urls.tokens_rest),
            Self::TokenCategory => urls.tokens_rest.to_string(), // path appended dynamically
            Self::TokenRecent => format!("{}/recent", urls.tokens_rest),
        }
    }

    /// Требует ли endpoint API key
    pub fn requires_api_key(&self) -> bool {
        // All endpoints require API key in Jupiter API v1 (since Oct 2025)
        true
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::Swap | Self::SwapInstructions => "POST",
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MINT ADDRESS REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Реестр популярных Solana token mint addresses
pub struct MintRegistry;

impl MintRegistry {
    // Major tokens
    pub const SOL: &'static str = "So11111111111111111111111111111111111111112";
    pub const USDC: &'static str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const USDT: &'static str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    pub const JUP: &'static str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";
    pub const RAY: &'static str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
    pub const ORCA: &'static str = "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE";

    // Stablecoins
    pub const USDH: &'static str = "USDH1SM1ojwWUga67PGrgFWUHibbjqMvuMaDkRJTgkX";
    pub const UXD: &'static str = "7kbnvuGBxxj8AG9qp8Scn56muWGaRaFqxg1FsRp3PaFT";

    // Liquid Staking Tokens
    pub const MSOL: &'static str = "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So";
    pub const STSOL: &'static str = "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj";
    pub const JITOSOL: &'static str = "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn";

    // Meme tokens
    pub const BONK: &'static str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
    pub const WIF: &'static str = "EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm";

    /// Convert symbol to mint address (fallible)
    pub fn symbol_to_mint(symbol: &str) -> Option<&'static str> {
        match symbol.to_uppercase().as_str() {
            "SOL" => Some(Self::SOL),
            "USDC" => Some(Self::USDC),
            "USDT" => Some(Self::USDT),
            "JUP" => Some(Self::JUP),
            "RAY" => Some(Self::RAY),
            "ORCA" => Some(Self::ORCA),
            "USDH" => Some(Self::USDH),
            "UXD" => Some(Self::UXD),
            "MSOL" => Some(Self::MSOL),
            "STSOL" => Some(Self::STSOL),
            "JITOSOL" => Some(Self::JITOSOL),
            "BONK" => Some(Self::BONK),
            "WIF" => Some(Self::WIF),
            _ => None,
        }
    }

    /// Get decimals for known tokens
    pub fn decimals(mint: &str) -> Option<u8> {
        match mint {
            // 9 decimals (SOL and LSTs)
            Self::SOL | Self::MSOL | Self::STSOL | Self::JITOSOL => Some(9),
            // 6 decimals (most tokens)
            Self::USDC | Self::USDT | Self::JUP | Self::RAY | Self::ORCA
            | Self::USDH | Self::UXD | Self::WIF => Some(6),
            // 5 decimals
            Self::BONK => Some(5),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMOUNT CONVERSION
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert human-readable amount to raw amount (u64)
///
/// # Example
/// ```ignore
/// let raw = to_raw_amount(1.5, 9); // 1.5 SOL = 1_500_000_000
/// ```
pub fn to_raw_amount(human_amount: f64, decimals: u8) -> u64 {
    (human_amount * 10f64.powi(decimals as i32)) as u64
}

/// Convert raw amount (u64) to human-readable amount
///
/// # Example
/// ```ignore
/// let human = from_raw_amount(1_500_000_000, 9); // 1.5 SOL
/// ```
pub fn from_raw_amount(raw_amount: u64, decimals: u8) -> f64 {
    raw_amount as f64 / 10f64.powi(decimals as i32)
}

/// Validate if string is a valid Solana mint address (Base58)
pub fn is_valid_mint_address(address: &str) -> bool {
    const BASE58: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

    // Check length (32-44 chars for Solana addresses)
    if address.len() < 32 || address.len() > 44 {
        return false;
    }

    // Check all chars are valid base58
    address.chars().all(|c| BASE58.contains(c))
}
