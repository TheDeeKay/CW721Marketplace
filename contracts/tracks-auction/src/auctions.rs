use cosmwasm_std::Storage;
use cw_storage_plus::Map;
use tracks_auction_api::api::TrackAuction;
use tracks_auction_api::error::AuctionResult;

// const NEXT_AUCTION_ID: Item<u64> = Item::new("next_auction_id");

const AUCTIONS_MAP: Map<u64, TrackAuction> = Map::new("auctions");

pub fn save_auction(storage: &mut dyn Storage, auction: TrackAuction) -> AuctionResult<()> {
    // TODO: provoke by tests
    // let next_auction_id = NEXT_AUCTION_ID.may_load(storage)?.unwrap_or_default();
    // NEXT_AUCTION_ID.save(storage, &(next_auction_id + 1)?;

    AUCTIONS_MAP.save(storage, 0, &auction)?;

    Ok(())
}

pub fn load_auctions(storage: &dyn Storage) -> AuctionResult<Vec<TrackAuction>> {
    // TODO: use limit and start_after

    Ok(AUCTIONS_MAP
        .may_load(storage, 0)?
        .into_iter()
        .collect::<Vec<TrackAuction>>())
}
