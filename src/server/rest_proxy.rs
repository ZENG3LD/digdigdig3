//! RestProxy gRPC service — unary REST calls forwarded through ExchangeHub.

use tonic::{Request, Response, Status};

use crate::core::types::{AccountType, ExchangeId, SymbolInput};
use crate::server::proto::{
    rest_proxy_server::RestProxy,
    GetKlinesRequest, GetKlinesResponse,
    GetOrderbookRequest, GetOrderbookResponse,
    GetTickerRequest, GetTickerResponse,
};
use crate::server::state::ServerState;

pub struct RestProxyService {
    pub state: ServerState,
}

#[tonic::async_trait]
impl RestProxy for RestProxyService {
    async fn get_ticker(
        &self,
        request: Request<GetTickerRequest>,
    ) -> Result<Response<GetTickerResponse>, Status> {
        let req = request.into_inner();
        let (id, account) = parse_exchange_account(&req.exchange, &req.account)?;
        let conn = self
            .state
            .hub
            .rest(id)
            .ok_or_else(|| Status::not_found(format!("{:?} not connected", id)))?;

        let ticker = conn
            .get_ticker(SymbolInput::Raw(&req.symbol), account)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let json = serde_json::to_vec(&ticker)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetTickerResponse { ticker_json: json }))
    }

    async fn get_klines(
        &self,
        request: Request<GetKlinesRequest>,
    ) -> Result<Response<GetKlinesResponse>, Status> {
        let req = request.into_inner();
        let (id, account) = parse_exchange_account(&req.exchange, &req.account)?;
        let conn = self
            .state
            .hub
            .rest(id)
            .ok_or_else(|| Status::not_found(format!("{:?} not connected", id)))?;

        let limit = if req.limit > 0 {
            Some(req.limit as u16)
        } else {
            None
        };

        let klines = conn
            .get_klines(
                SymbolInput::Raw(&req.symbol),
                &req.interval,
                limit,
                account,
                None,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let json = serde_json::to_vec(&klines)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetKlinesResponse { klines_json: json }))
    }

    async fn get_orderbook(
        &self,
        request: Request<GetOrderbookRequest>,
    ) -> Result<Response<GetOrderbookResponse>, Status> {
        let req = request.into_inner();
        let (id, account) = parse_exchange_account(&req.exchange, &req.account)?;
        let conn = self
            .state
            .hub
            .rest(id)
            .ok_or_else(|| Status::not_found(format!("{:?} not connected", id)))?;

        let depth = if req.depth > 0 {
            Some(req.depth as u16)
        } else {
            None
        };

        let book = conn
            .get_orderbook(SymbolInput::Raw(&req.symbol), depth, account)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let json = serde_json::to_vec(&book)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetOrderbookResponse {
            orderbook_json: json,
        }))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn parse_exchange_id(s: &str) -> Result<ExchangeId, Status> {
    serde_json::from_str(&format!("\"{}\"", s))
        .map_err(|_| Status::invalid_argument(format!("unknown exchange: {}", s)))
}

fn parse_account_type(s: &str) -> Result<AccountType, Status> {
    serde_json::from_str::<AccountType>(&format!("\"{}\"", s))
        .map_err(|_| Status::invalid_argument(format!("unknown account: {}", s)))
}

fn parse_exchange_account(
    exchange: &str,
    account: &str,
) -> Result<(ExchangeId, AccountType), Status> {
    let id = parse_exchange_id(exchange)?;
    let acct = if account.is_empty() {
        AccountType::Spot
    } else {
        parse_account_type(account)?
    };
    Ok((id, acct))
}
