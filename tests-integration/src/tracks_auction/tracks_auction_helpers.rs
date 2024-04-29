use crate::helpers::ADMIN;
use cosmwasm_std::Addr;
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, ContractWrapper, Executor, IntoAddr};
use tracks_auction_api::api::PriceAssetUnchecked;
use tracks_auction_api::msg::InstantiateMsg;

pub fn store_tracks_auction_code(app: &mut App) -> u64 {
    app.store_code(Box::new(ContractWrapper::new(
        tracks_auction::contract::execute,
        tracks_auction::contract::instantiate,
        tracks_auction::contract::query,
    )))
}

pub fn instantiate_tracks_auction(
    app: &mut App,
    code_id: u64,
    whitelisted_nft: String,
    price_asset: PriceAssetUnchecked,
) -> AnyResult<Addr> {
    let msg = InstantiateMsg {
        whitelisted_nft,
        price_asset,
    };

    app.instantiate_contract(
        code_id,
        ADMIN.into_addr(),
        &msg,
        &[],
        "Tracks auction",
        Some(ADMIN.to_string()),
    )
}

pub fn store_and_instantiate_tracks_auction(
    app: &mut App,
    whitelisted_nft: String,
    price_asset: PriceAssetUnchecked,
) -> AnyResult<(u64, Addr)> {
    let code_id = store_tracks_auction_code(app);
    let addr = instantiate_tracks_auction(app, code_id, whitelisted_nft, price_asset);

    addr.map(|address| (code_id, address))
}
