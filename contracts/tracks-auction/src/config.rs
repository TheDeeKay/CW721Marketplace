use cosmwasm_std::Storage;
use cw_storage_plus::Item;
use tracks_auction_api::api::Config;
use tracks_auction_api::error::AuctionResult;

const CONFIG: Item<Config> = Item::new("config");

pub fn load_config(storage: &dyn Storage) -> AuctionResult<Config> {
    let config = CONFIG.load(storage)?;
    Ok(config)
}

pub fn save_config(storage: &mut dyn Storage, config: &Config) -> AuctionResult<()> {
    CONFIG.save(storage, config)?;
    Ok(())
}
