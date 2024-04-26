use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct Config {
    pub whitelisted_nft: Addr,
}

#[cw_serde]
pub struct TrackAuction {
    /// The address that submitted this auction. This will be the address that receives the
    /// funds, or the NFT (if the auction fails).
    pub submitter: Addr,
    pub track_token_id: String,
    pub minimum_bid_amount: Uint128,
}

#[cw_serde]
pub struct AuctionsResponse {
    pub auctions: Vec<TrackAuction>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
