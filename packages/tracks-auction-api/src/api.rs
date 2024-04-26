use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct TrackAuction {
    pub track_nft_contract: Addr,
    pub track_token_id: String,
    pub minimum_bid_amount: Uint128,
}

#[cw_serde]
pub struct AuctionsResponse {
    pub auctions: Vec<TrackAuction>,
}

#[cw_serde]
pub struct NftWhitelistResponse {
    pub nft_whitelist: Vec<Addr>,
}
