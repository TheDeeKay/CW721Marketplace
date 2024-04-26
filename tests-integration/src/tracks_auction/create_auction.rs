use crate::cw721_tracks::cw721_tracks_helpers::store_and_instantiate_cw721_tracks;
use crate::helpers::{ADMIN, USER1};
use crate::tracks_auction::tracks_auction_helpers::store_and_instantiate_tracks_auction;
use cosmwasm_std::to_json_binary;
use cw721::Cw721ExecuteMsg::SendNft;
use cw721_tracks_api::api::TrackMetadata;
use cw_multi_test::{App, Executor, IntoAddr};
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;

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
