use crate::helpers::ADMIN;
use cosmwasm_std::Addr;
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, ContractWrapper, Executor, IntoAddr};

pub fn store_cw721_tracks_code(app: &mut App) -> u64 {
    app.store_code(Box::new(ContractWrapper::new(
        cw721_tracks::contract::execute,
        cw721_tracks::contract::instantiate,
        cw721_tracks::contract::query,
    )))
}

// TODO: remove minter param
pub fn instantiate_cw721_tracks(app: &mut App, code_id: u64, minter: &str) -> AnyResult<Addr> {
    let msg = cw721_tracks_api::msg::InstantiateMsg {
        name: "CW721 tracks".to_string(),
        symbol: "TRKS".to_string(),
        minter: minter.into_addr().to_string(), // TODO: this has to change
    };

    app.instantiate_contract(
        code_id,
        ADMIN.into_addr(),
        &msg,
        &[],
        "Tracks auction",
        Some(ADMIN.into_addr().to_string()),
    )
}

// TODO: remove minter param
pub fn store_and_instantiate_cw721_tracks(app: &mut App, minter: &str) -> AnyResult<(u64, Addr)> {
    let code_id = store_cw721_tracks_code(app);
    let addr = instantiate_cw721_tracks(app, code_id, minter);

    addr.map(|address| (code_id, address))
}
