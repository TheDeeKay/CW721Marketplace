use crate::api::PriceAsset::Cw20;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, BlockInfo, StdResult, Uint128};
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
pub enum AuctionStatus {
    Active,
    Resolved,
    Canceled,
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
    pub status: AuctionStatus,
    pub created_at: BlockInfo,
    pub duration: Duration,
    /// ID of the auction posting
    pub id: AuctionId,
    /// The address that submitted this auction. This will be the address that receives the
    /// funds, or the NFT (if the auction fails).
    pub creator: Addr,
    /// NFT contract to which the token representing this track belongs.
    pub nft_contract: Addr,
    /// ID of the NFT token representing this track.
    pub track_token_id: String,
    /// Minimum initial bid that will be accepted.
    pub minimum_bid_amount: Uint128,
    /// Asset in which the price is denominated.
    pub price_asset: PriceAsset,
    /// Last (highest) bid, if any.
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
pub enum PriceAssetUnchecked {
    Native { denom: String },
    Cw20 { contract: String },
}

impl PriceAssetUnchecked {
    pub fn check(&self, api: &dyn Api) -> StdResult<PriceAsset> {
        match self {
            PriceAssetUnchecked::Native { denom } => Ok(PriceAsset::native(denom)),
            PriceAssetUnchecked::Cw20 { contract } => {
                Ok(PriceAsset::cw20(api.addr_validate(contract)?))
            }
        }
    }
}

#[cw_serde]
pub enum PriceAsset {
    Native { denom: String },
    Cw20 { contract: Addr },
}

impl PriceAsset {
    pub fn native(denom: impl Into<String>) -> Self {
        Native {
            denom: denom.into(),
        }
    }

    pub fn cw20(contract: Addr) -> Self {
        Cw20 { contract }
    }

    pub fn to_asset_info(&self) -> AssetInfo {
        match self {
            Native { denom } => AssetInfo::native(denom),
            Cw20 { contract } => AssetInfo::cw20(contract.clone()),
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
