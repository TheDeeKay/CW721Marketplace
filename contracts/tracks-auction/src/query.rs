use crate::auctions::load_auctions;
use crate::config::load_config;
use cosmwasm_std::Deps;
use tracks_auction_api::api::{AuctionsResponse, ConfigResponse};
use tracks_auction_api::error::AuctionResult;

pub fn query_config(deps: Deps) -> AuctionResult<ConfigResponse> {
    let config = load_config(deps.storage)?;
    Ok(ConfigResponse { config })
}

// TODO: include limit and start_after (we'll probably need an ID inside the Auction structure then
pub fn query_auctions(deps: Deps) -> AuctionResult<AuctionsResponse> {
    let auctions = load_auctions(deps.storage)?;
    Ok(AuctionsResponse { auctions })
}
