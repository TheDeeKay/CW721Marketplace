use cosmwasm_std::{Addr, Storage};
use cw_storage_plus::Map;
use tracks_auction_api::error::AuctionResult;

const CW721_WHITELIST: Map<Addr, ()> = Map::new("cw721_whitelist");

pub fn is_cw721_whitelisted(storage: &dyn Storage, cw721: Addr) -> AuctionResult<bool> {
    Ok(CW721_WHITELIST.has(storage, cw721))
}

pub fn add_whitelisted_cw721(storage: &mut dyn Storage, cw721: Addr) -> AuctionResult<()> {
    CW721_WHITELIST.save(storage, cw721, &())?;
    Ok(())
}
