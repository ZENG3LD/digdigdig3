# GMX V2 Authentication Levels and Rate Limits

Source date: 2026-03-11
Official docs: https://docs.gmx.io/
GitHub: https://github.com/gmx-io/gmx-synthetics

---

## Overview

GMX V2 is a decentralized protocol. Authentication is NOT based on API keys or centralized login credentials. "Authentication" in GMX V2 means:

1. **For reads**: No auth required for REST and Subsquid endpoints
2. **For writes**: Ethereum private key / wallet signature (standard EVM transaction signing)

---

## Authentication for Read Endpoints

### REST API (gmxinfra.io)

**Authentication required: NONE**

All read endpoints on `arbitrum-api.gmxinfra.io`, `avalanche-api.gmxinfra.io`, etc. are publicly accessible:

```
GET https://arbitrum-api.gmxinfra.io/markets/info        # No auth
GET https://arbitrum-api.gmxinfra.io/prices/tickers      # No auth
GET https://arbitrum-api.gmxinfra.io/tokens              # No auth
GET https://arbitrum-api.gmxinfra.io/signed_prices/latest # No auth
GET https://arbitrum-api.gmxinfra.io/prices/candles       # No auth
GET https://arbitrum-api.gmxinfra.io/glvs/info           # No auth
GET https://arbitrum-api.gmxinfra.io/ping                # No auth
```

No headers required. No API keys. No OAuth. Standard HTTP GET requests.

### Subsquid GraphQL API

**Authentication required: NONE**

The Subsquid endpoints are open public GraphQL APIs:

```
POST https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql
POST https://gmx.squids.live/gmx-synthetics-avalanche:prod/api/graphql
```

Standard GraphQL POST with `Content-Type: application/json` body. No authorization headers required.

### On-Chain Reads (Reader contract)

**Authentication required: NONE**

EVM `eth_call` to the Reader contract is a read-only operation. No transaction signing. No private key. Anyone can call `Reader.getAccountPositions()` for any address.

---

## Authentication for Write Operations

**Authentication: Ethereum Wallet Signature (EIP-191 / EIP-712)**

All writes to GMX V2 are on-chain Ethereum transactions. The authentication mechanism is the standard Ethereum transaction signing model:

- The transaction `from` field must match the authorized address
- Transactions are signed with the account's ECDSA private key (secp256k1)
- Signature is embedded in the transaction itself (v, r, s fields)

### Who Can Place Orders

**Any Ethereum address (EOA or smart contract) can create orders by calling `ExchangeRouter.createOrder()`.**

There is **no whitelist**, **no API key**, and **no off-chain authorization** required for standard order placement.

The only restriction is practical: gas fees and execution fees (ETH) must be funded in the wallet.

### Smart Contract Interactions

Smart contracts can interact with GMX V2 identically to EOAs:
- Call `ExchangeRouter.createOrder()` with `callbackContract = address(0)`
- This requires no whitelisting or governance approval
- Setting `callbackContract` to a non-zero address (for custom callbacks) does NOT require whitelisting either — the `ROUTER_PLUGIN` restriction applies only to legacy proxy-based auth, which is no longer needed with the current ExchangeRouter

Source: GMX governance thread confirming "no whitelisting is needed" as of 2024.

### ExchangeRouter Access Control

The ExchangeRouter enforces:
- Order can only be updated/cancelled by the `receiver` address set at creation
- Liquidations are executed by authorized keeper bots, not arbitrary callers
- The `autoCancel` feature is owner-controlled

---

## Rate Limits

### REST API (gmxinfra.io)

**Rate limits: NOT DOCUMENTED in official sources.**

Observations:
- The API is a public infrastructure endpoint run by the GMX team
- No official rate limit policy is published in the docs
- No `X-RateLimit-*` response headers are documented
- The SDK and interface code poll these endpoints continuously in production without documented limits
- Recommend: use reasonable polling intervals (1-5 seconds for prices, longer for market info)

For high-frequency use, consider:
- Using the fallback endpoints to distribute load
- Fetching on-chain data directly via RPC instead of REST

### Subsquid GraphQL API

**Rate limits: NOT DOCUMENTED in official sources.**

Subsquid's general infrastructure applies standard query complexity limits. No per-key or per-IP limits are documented for the GMX endpoints.

### On-Chain RPC (Reader contract calls)

Rate limits are determined by the RPC provider, not GMX:
- Public Arbitrum RPC: standard `eth_call` rate limits apply
- Use a private RPC (e.g., Infura, Alchemy, QuickNode) for production use
- No GMX-specific rate limits on Reader calls

---

## Whitelisting / Special Access

### Standard Trading

**No whitelist required.**

Any address can:
- Call `ExchangeRouter.createOrder()`, `updateOrder()`, `cancelOrder()`
- Call `ExchangeRouter.createDeposit()`, `createWithdrawal()`
- Read all public data via Reader, REST, or Subsquid

### Keeper Network

Keepers (bots that execute orders) must be **authorized** by GMX governance. This is NOT required for traders. The GMX team operates the official keeper network.

Custom keeper development is possible but requires understanding the signed price oracle system. Keepers call `OrderHandler.executeOrder()` with signed prices from the oracle network.

### Oracle Price Signing

Oracles that sign prices for the `signed_prices/latest` feed are maintained by the GMX team using Chainlink Data Streams (realtimeFeed2). Third parties cannot inject prices into this feed without authorization.

---

## RPC Endpoint Requirements

To interact with GMX V2 on-chain, you need an Arbitrum (or Avalanche) JSON-RPC endpoint:

| Network | Chain ID | Public RPC |
|---------|----------|-----------|
| Arbitrum One | 42161 | `https://arb1.arbitrum.io/rpc` |
| Avalanche C-Chain | 43114 | `https://api.avax.network/ext/bc/C/rpc` |

For production use, a private RPC endpoint is recommended.

---

## SDK Authentication

The `@gmx-io/sdk` configuration accepts a `walletClient` (viem):

```typescript
const sdk = new GmxSdk({
  chainId: 42161,
  rpcUrl: "https://arb1.arbitrum.io/rpc",
  oracleUrl: "https://arbitrum-api.gmxinfra.io",
  walletClient: walletClient,    // Viem WalletClient with private key signer
  subsquidUrl: "https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql"
});
```

The wallet client handles all transaction signing. Read-only SDK usage does not require a walletClient.

---

## API Updates Channel

GMX publishes contract and API updates to:
- Telegram: `@GMX_Technical_Announcements`
- GitHub releases: https://github.com/gmx-io/gmx-synthetics/releases

Subscribe for breaking changes to contract addresses, API endpoints, or schema changes.

---

## Summary Table

| Operation | Auth Required | Mechanism |
|-----------|--------------|-----------|
| REST GET endpoints | None | Open HTTP |
| Subsquid GraphQL queries | None | Open GraphQL POST |
| Reader contract eth_call | None | Open EVM read |
| Create/update/cancel order | Yes | Ethereum tx signature |
| Create deposit/withdrawal | Yes | Ethereum tx signature |
| Execute orders (keeper) | Yes (keeper role) | Authorized keeper only |
| Sign oracle prices | Yes (oracle role) | GMX team / Chainlink |

---

## Sources

- [API category | GMX Docs](https://docs.gmx.io/docs/category/api/)
- [REST V2 | GMX Docs](https://docs.gmx.io/docs/api/rest-v2/)
- [Subsquid | GMX Docs](https://docs.gmx.io/docs/api/subsquid/)
- [SDK V2 | GMX Docs](https://docs.gmx.io/docs/api/sdk-v2/)
- [Contracts V2 | GMX Docs](https://docs.gmx.io/docs/api/contracts-v2/)
- [Updates and Support | GMX Docs](https://docs.gmx.io/docs/api/updates-support/)
- [GitHub - gmx-io/gmx-synthetics](https://github.com/gmx-io/gmx-synthetics)
- [GMX Governance - Whitelist Thread](https://gov.gmx.io/t/whitelist-request-aifuturestradingbot-arbitrum-owned-by-my-eoa/4879/)
- [QuickNode GMX Builder Guide](https://www.quicknode.com/builders-guide/tools/gmx)
