use cosmwasm_std::Order::Ascending;
use cosmwasm_std::{Addr, StdResult, Storage};
use cw_storage_plus::Map;
use tracks_auction_api::error::AuctionResult;

// TODO: perhaps should sit in a more appropriately named file
const CW721_WHITELIST: Map<Addr, ()> = Map::new("cw721_whitelist");

pub fn is_cw721_whitelisted(storage: &dyn Storage, cw721: Addr) -> AuctionResult<bool> {
    Ok(CW721_WHITELIST.has(storage, cw721))
}

pub fn add_whitelisted_cw721(storage: &mut dyn Storage, cw721: Addr) -> AuctionResult<()> {
    CW721_WHITELIST.save(storage, cw721, &())?;
    Ok(())
}

// TODO: this will need a limit and start_after
pub fn load_whitelisted_nfts(storage: &dyn Storage) -> AuctionResult<Vec<Addr>> {
    let nfts = CW721_WHITELIST
        .range(storage, None, None, Ascending)
        .map(|res| res.map(|(nft, _)| nft))
        .collect::<StdResult<Vec<Addr>>>()?;

    Ok(nfts)
}
