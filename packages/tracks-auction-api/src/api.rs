use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

pub type AuctionId = u64;

#[cw_serde]
pub struct Config {
    pub whitelisted_nft: Addr,
    // TODO: add PriceAsset here
    pub price_asset: PriceAsset,
}

#[cw_serde]
pub struct Bid {
    pub amount: Uint128,
    pub asset: PriceAsset,
    pub bidder: Addr,
}

#[cw_serde]
pub struct TrackAuction {
    /// ID of the auction posting
    pub id: AuctionId,
    /// The address that submitted this auction. This will be the address that receives the
    /// funds, or the NFT (if the auction fails).
    pub submitter: Addr,
    // TODO: add NFT contract here
    /// ID of the NFT token representing this track.
    pub track_token_id: String,
    pub minimum_bid_amount: Uint128,
    pub price_asset: PriceAsset,
    pub active_bid: Option<Bid>,
}

#[cw_serde]
pub enum PriceAsset {
    Native { denom: String },
    // TODO: add CW20 here as well (once we do, we need to split this into PriceAssetUnchecked as well)
}

impl PriceAsset {
    pub fn native(denom: impl Into<String>) -> Self {
        PriceAsset::Native {
            denom: denom.into(),
        }
    }
}

#[cw_serde]
pub struct AuctionsResponse {
    pub auctions: Vec<TrackAuction>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
