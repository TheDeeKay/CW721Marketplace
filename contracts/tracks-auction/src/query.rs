use crate::auctions::{load_auction, load_auctions};
use crate::config::load_config;
use cosmwasm_std::Deps;
use tracks_auction_api::api::{AuctionId, AuctionResponse, AuctionsResponse, ConfigResponse};
use tracks_auction_api::error::AuctionError::AuctionIdNotFound;
use tracks_auction_api::error::AuctionResult;

pub fn query_config(deps: Deps) -> AuctionResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_auction(deps: Deps, id: AuctionId) -> AuctionResult<AuctionResponse> {
    let auction = load_auction(deps.storage, id)?.ok_or(AuctionIdNotFound)?;

    Ok(AuctionResponse { auction })
}

pub fn query_auctions(
    deps: Deps,
    start_after: Option<AuctionId>,
    limit: Option<u32>,
) -> AuctionResult<AuctionsResponse> {
    let auctions = load_auctions(deps.storage, start_after, limit)?;
    Ok(AuctionsResponse { auctions })
}
