use crate::config::load_config;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, BlockInfo, StdError, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Item, Map};
use cw_utils::Duration;
use tracks_auction_api::api::{AuctionId, AuctionStatus, Bid, TrackAuction};
use tracks_auction_api::error::AuctionError::AuctionIdNotFound;
use tracks_auction_api::error::AuctionResult;
use AuctionStatus::{Active, Canceled, Resolved};

const DEFAULT_AUCTIONS_QUERY_LIMIT: u32 = 20;
const MAX_AUCTIONS_QUERY_LIMIT: u32 = 100;

const NEXT_AUCTION_ID: Item<u64> = Item::new("next_auction_id");

const AUCTIONS_MAP: Map<u64, TrackAuction> = Map::new("auctions");

pub struct CreateAuctionData {
    pub duration: Duration,
    pub creator: Addr,
    pub nft_contract: Addr,
    pub track_token_id: String,
    pub minimum_bid_amount: Uint128,
    pub buyout_price: Option<Uint128>,
}

pub fn save_new_auction(
    storage: &mut dyn Storage,
    current_block: BlockInfo,
    auction_data: CreateAuctionData,
) -> AuctionResult<AuctionId> {
    let next_auction_id = NEXT_AUCTION_ID.may_load(storage)?.unwrap_or_default();
    NEXT_AUCTION_ID.save(storage, &(next_auction_id + 1))?;

    let config = load_config(storage)?;

    AUCTIONS_MAP.save(
        storage,
        next_auction_id,
        &TrackAuction {
            status: Active,
            created_at: current_block,
            duration: auction_data.duration,
            id: next_auction_id,
            creator: auction_data.creator,
            nft_contract: auction_data.nft_contract,
            track_token_id: auction_data.track_token_id,
            minimum_bid_amount: auction_data.minimum_bid_amount,
            price_asset: config.price_asset,
            active_bid: None,
            buyout_price: auction_data.buyout_price,
        },
    )?;

    Ok(next_auction_id)
}

/// Updates active bid on the given auction ID.
/// Returns last active bid, or None if no previous bid on this auction existed.
pub fn update_active_bid(
    storage: &mut dyn Storage,
    auction_id: AuctionId,
    new_active_bid: Bid,
) -> AuctionResult<Option<Bid>> {
    let auction = load_auction(storage, auction_id)?.ok_or(AuctionIdNotFound)?;

    // TODO: also store the last bid for historical reasons?

    AUCTIONS_MAP.save(
        storage,
        auction_id,
        &TrackAuction {
            active_bid: Some(new_active_bid),
            ..auction
        },
    )?;

    Ok(auction.active_bid)
}

pub fn update_auction_status(
    storage: &mut dyn Storage,
    auction_id: AuctionId,
    new_status: AuctionStatus,
) -> AuctionResult<()> {
    let auction = load_auction(storage, auction_id)?.ok_or(AuctionIdNotFound)?;

    // only Active status can be changed, others are final
    match auction.status {
        Resolved | Canceled => {
            return Err(StdError::generic_err(
                "invalid state - can only change status of an active auction",
            )
            .into())
        }
        Active => {
            // no-op, can be changed
        }
    }

    AUCTIONS_MAP.save(
        storage,
        auction_id,
        &TrackAuction {
            status: new_status,
            ..auction
        },
    )?;

    Ok(())
}

pub fn load_auctions(
    storage: &dyn Storage,
    start_after: Option<AuctionId>,
    limit: Option<u32>,
) -> AuctionResult<Vec<TrackAuction>> {
    let start_after = start_after.map(Bound::exclusive);
    let limit = limit
        .unwrap_or(DEFAULT_AUCTIONS_QUERY_LIMIT)
        .min(MAX_AUCTIONS_QUERY_LIMIT);

    Ok(AUCTIONS_MAP
        .range(storage, start_after, None, Ascending)
        .take(limit as usize)
        .map(|res| res.map(|(_, auction)| auction))
        .collect::<StdResult<Vec<TrackAuction>>>()?)
}

pub fn load_auction(storage: &dyn Storage, id: AuctionId) -> AuctionResult<Option<TrackAuction>> {
    Ok(AUCTIONS_MAP.may_load(storage, id)?)
}
