use crate::config::load_config;
use cosmwasm_std::{Addr, Storage, Uint128};
use cw_storage_plus::Map;
use tracks_auction_api::api::{AuctionId, TrackAuction};
use tracks_auction_api::error::AuctionResult;

// const NEXT_AUCTION_ID: Item<u64> = Item::new("next_auction_id");

const AUCTIONS_MAP: Map<u64, TrackAuction> = Map::new("auctions");

pub fn save_auction(
    storage: &mut dyn Storage,
    submitter: Addr,
    track_token_id: String,
    minimum_bid_amount: Uint128,
) -> AuctionResult<()> {
    // TODO: provoke by tests
    // let next_auction_id = NEXT_AUCTION_ID.may_load(storage)?.unwrap_or_default();
    // NEXT_AUCTION_ID.save(storage, &(next_auction_id + 1)?;

    let config = load_config(storage)?;

    AUCTIONS_MAP.save(
        storage,
        0,
        &TrackAuction {
            id: 0,
            submitter,
            track_token_id,
            minimum_bid_amount,
            price_asset: config.price_asset,
        },
    )?;

    Ok(())
}

pub fn load_auctions(storage: &dyn Storage) -> AuctionResult<Vec<TrackAuction>> {
    // TODO: use limit and start_after

    // TODO: use range here, provoke by tests
    Ok(AUCTIONS_MAP
        .may_load(storage, 0)?
        .into_iter()
        .collect::<Vec<TrackAuction>>())
}

pub fn load_auction(storage: &dyn Storage, id: AuctionId) -> AuctionResult<Option<TrackAuction>> {
    Ok(AUCTIONS_MAP.may_load(storage, id)?)
}
