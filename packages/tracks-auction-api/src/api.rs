use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Uint128};
use cw_asset::AssetInfo;
use cw_utils::Duration;
use cw_utils::Duration::{Height, Time};
use std::ops::Add;
use PriceAsset::Native;

pub type AuctionId = u64;

#[cw_serde]
pub struct Config {
    pub whitelisted_nft: Addr,
    pub price_asset: PriceAsset,
}

#[cw_serde]
pub struct Bid {
    pub amount: Uint128,
    pub asset: PriceAsset,
    pub bidder: Addr,
    pub posted_at: BlockInfo,
}

#[cw_serde]
pub struct TrackAuction {
    pub created_at: BlockInfo,
    pub duration: Duration,
    /// ID of the auction posting
    pub id: AuctionId,
    /// The address that submitted this auction. This will be the address that receives the
    /// funds, or the NFT (if the auction fails).
    pub submitter: Addr,
    pub nft_contract: Addr,
    /// ID of the NFT token representing this track.
    pub track_token_id: String,
    pub minimum_bid_amount: Uint128,
    pub price_asset: PriceAsset,
    pub active_bid: Option<Bid>,
}

impl TrackAuction {
    /// Calculate what the minimum amount should be for the next bid
    pub fn minimum_next_bid_amount(&self) -> Uint128 {
        match &self.active_bid {
            None => self.minimum_bid_amount,
            Some(bid) => bid.amount.add(Uint128::from(1u8)),
        }
    }

    pub fn has_ended(&self, current_block: &BlockInfo) -> bool {
        match self.duration {
            Height(height) => current_block.height > self.created_at.height + height,
            Time(seconds) => current_block.time > self.created_at.time.plus_seconds(seconds),
        }
    }
}

#[cw_serde]
pub enum PriceAsset {
    Native { denom: String },
    // TODO: add CW20 here as well (once we do, we need to split this into PriceAssetUnchecked as well)
}

impl PriceAsset {
    pub fn native(denom: impl Into<String>) -> Self {
        Native {
            denom: denom.into(),
        }
    }

    pub fn to_asset_info(&self) -> AssetInfo {
        match self {
            Native { denom } => AssetInfo::native(denom),
        }
    }
}

#[cw_serde]
pub struct AuctionResponse {
    pub auction: TrackAuction,
}

#[cw_serde]
pub struct AuctionsResponse {
    pub auctions: Vec<TrackAuction>,
}

#[cw_serde]
pub struct ConfigResponse {
    pub config: Config,
}
