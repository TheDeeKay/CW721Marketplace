use cosmwasm_std::{to_json_binary, Addr};
use cw721::Cw721ExecuteMsg::SendNft;
use cw721_tracks_api::api::TrackMetadata;
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, ContractWrapper, Executor, IntoAddr};
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;

const ADMIN: &str = "admin";

const USER1: &str = "user1";

#[test]
fn create_auction_only_possible_with_whitelisted_nft_contract() -> anyhow::Result<()> {
    let mut app = App::default();

    let (_, tracks_auction) = store_and_instantiate_tracks_auction(&mut app)?;

    let (_, cw721_tracks) = store_and_instantiate_cw721_tracks(&mut app, USER1)?;

    // TODO: extract minting function
    app.execute_contract(
        USER1.into_addr(),
        cw721_tracks.clone(),
        &cw721_tracks_api::msg::ExecuteMsg::Mint {
            token_id: "1".to_string(),
            owner: USER1.to_string(),
            token_uri: None,
            extension: TrackMetadata {
                artist_name: "Boden".to_string(),
                album: None,
                track_name: "Debt Spiral".to_string(),
                audio_track_url: "https://www.usdebtclock.org/".to_string(),
            },
        },
        &vec![],
    )?;

    let result = app.execute_contract(
        ADMIN.into_addr(),
        cw721_tracks,
        &SendNft {
            contract: tracks_auction.to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&CreateAuction {})?,
        },
        &vec![],
    );

    assert!(result.is_err());

    Ok(())
}

fn store_tracks_auction_code(app: &mut App) -> u64 {
    app.store_code(Box::new(ContractWrapper::new(
        tracks_auction::contract::execute,
        tracks_auction::contract::instantiate,
        tracks_auction::contract::query,
    )))
}

fn instantiate_tracks_auction(app: &mut App, code_id: u64) -> AnyResult<Addr> {
    let msg = tracks_auction_api::msg::InstantiateMsg {};

    app.instantiate_contract(
        code_id,
        ADMIN.into_addr(),
        &msg,
        &[],
        "Tracks auction",
        Some(ADMIN.to_string()),
    )
}

fn store_and_instantiate_tracks_auction(app: &mut App) -> AnyResult<(u64, Addr)> {
    let code_id = store_tracks_auction_code(app);
    let addr = instantiate_tracks_auction(app, code_id);

    addr.map(|address| (code_id, address))
}

fn store_cw721_tracks_code(app: &mut App) -> u64 {
    app.store_code(Box::new(ContractWrapper::new(
        cw721_tracks::contract::execute,
        cw721_tracks::contract::instantiate,
        cw721_tracks::contract::query,
    )))
}

// TODO: remove minter param
fn instantiate_cw721_tracks(app: &mut App, code_id: u64, minter: &str) -> AnyResult<Addr> {
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
        Some(ADMIN.to_string()),
    )
}

// TODO: remove minter param
fn store_and_instantiate_cw721_tracks(app: &mut App, minter: &str) -> AnyResult<(u64, Addr)> {
    let code_id = store_cw721_tracks_code(app);
    let addr = instantiate_cw721_tracks(app, code_id, minter);

    addr.map(|address| (code_id, address))
}
