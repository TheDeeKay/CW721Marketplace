use crate::config::load_config;
use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use cw_utils::Duration;
use tracks_auction_api::api::{AuctionId, Bid, TrackAuction};
use tracks_auction_api::error::AuctionError::AuctionIdNotFound;
use tracks_auction_api::error::AuctionResult;

const NEXT_AUCTION_ID: Item<u64> = Item::new("next_auction_id");

const AUCTIONS_MAP: Map<u64, TrackAuction> = Map::new("auctions");

pub fn save_new_auction(
    storage: &mut dyn Storage,
    current_block: BlockInfo,
    duration: Duration,
    submitter: Addr,
    track_token_id: String,
    minimum_bid_amount: Uint128,
) -> AuctionResult<()> {
    let next_auction_id = NEXT_AUCTION_ID.may_load(storage)?.unwrap_or_default();
    NEXT_AUCTION_ID.save(storage, &(next_auction_id + 1))?;

    let config = load_config(storage)?;

    AUCTIONS_MAP.save(
        storage,
        next_auction_id,
        &TrackAuction {
            created_at: current_block,
            duration,
            id: next_auction_id,
            submitter,
            track_token_id,
            minimum_bid_amount,
            price_asset: config.price_asset,
            active_bid: None,
        },
    )?;

    Ok(())
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

pub fn load_auctions(storage: &dyn Storage) -> AuctionResult<Vec<TrackAuction>> {
    // TODO: use limit and start_after

    Ok(AUCTIONS_MAP
        .range(storage, None, None, Ascending)
        .map(|res| res.map(|(_, auction)| auction))
        .collect::<StdResult<Vec<TrackAuction>>>()?)
}

pub fn load_auction(storage: &dyn Storage, id: AuctionId) -> AuctionResult<Option<TrackAuction>> {
    Ok(AUCTIONS_MAP.may_load(storage, id)?)
}
