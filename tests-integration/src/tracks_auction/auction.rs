use crate::cw721_tracks::cw721_tracks_helpers::{mint_nft, store_and_instantiate_cw721_tracks};
use crate::helpers::{UATOM, USER1};
use crate::tracks_auction::tracks_auction_helpers::store_and_instantiate_tracks_auction;
use cosmwasm_std::{to_json_binary, Uint128};
use cw721::Cw721ExecuteMsg::SendNft;
use cw721_tracks_api::api::TrackMetadata;
use cw_multi_test::{App, Executor, IntoAddr};
use cw_utils::Duration;
use tracks_auction_api::api::{AuctionStatus, AuctionsResponse, PriceAsset, TrackAuction};
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;
use tracks_auction_api::msg::QueryMsg::Auctions;
use AuctionStatus::Active;

// TODO: this is duplicated by unit tests - probably remove
#[test]
fn create_auction_only_possible_with_whitelisted_nft_contract() -> anyhow::Result<()> {
    let mut app = App::default();

    let (_, cw721_tracks) = store_and_instantiate_cw721_tracks(&mut app)?;

    let whitelisted_nft = "another_contract";
    assert_ne!(whitelisted_nft.to_string(), cw721_tracks.to_string());

    let (_, tracks_auction) = store_and_instantiate_tracks_auction(
        &mut app,
        whitelisted_nft.to_string(),
        PriceAsset::native(UATOM),
    )?;

    mint_nft(
        &mut app,
        cw721_tracks.clone(),
        USER1,
        "1",
        None,
        TrackMetadata {
            artist_name: "Boden".to_string(),
            album: None,
            track_name: "Debt Spiral".to_string(),
            audio_track_url: "https://www.usdebtclock.org/".to_string(),
        },
    )?;

    let result = app.execute_contract(
        USER1.into_addr(),
        cw721_tracks,
        &SendNft {
            contract: tracks_auction.to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&CreateAuction {
                duration: Duration::Time(200),
                minimum_bid_amount: Uint128::zero(),
            })?,
        },
        &vec![],
    );

    assert!(result.is_err());

    Ok(())
}

// TODO: this is duplicated by unit tests, and should probably expand in scope to keep making sense here too
#[test]
fn create_auction_saves_it_with_relevant_data() -> anyhow::Result<()> {
    let mut app = App::default();

    let (_, cw721_tracks) = store_and_instantiate_cw721_tracks(&mut app)?;

    let (_, tracks_auction) = store_and_instantiate_tracks_auction(
        &mut app,
        cw721_tracks.to_string(),
        PriceAsset::native(UATOM),
    )?;

    let track_token_id = "first_track";

    mint_nft(
        &mut app,
        cw721_tracks.clone(),
        USER1,
        track_token_id,
        None,
        TrackMetadata {
            artist_name: "Boden".to_string(),
            album: None,
            track_name: "Debt Spiral".to_string(),
            audio_track_url: "https://www.usdebtclock.org/".to_string(),
        },
    )?;

    let duration = Duration::Time(200);

    app.execute_contract(
        USER1.into_addr(),
        cw721_tracks.clone(),
        &SendNft {
            contract: tracks_auction.to_string(),
            token_id: track_token_id.to_string(),
            msg: to_json_binary(&CreateAuction {
                duration: duration.clone(),
                minimum_bid_amount: 4u128.into(),
            })?,
        },
        &vec![],
    )?;

    let response: AuctionsResponse = app
        .wrap()
        .query_wasm_smart(tracks_auction.clone(), &Auctions {})?;

    assert_eq!(
        response.auctions,
        vec![TrackAuction {
            status: Active,
            created_at: app.block_info(),
            duration,
            id: 0,
            submitter: USER1.into_addr(),
            nft_contract: cw721_tracks,
            track_token_id: track_token_id.to_string(),
            minimum_bid_amount: 4u128.into(),
            price_asset: PriceAsset::native(UATOM),
            active_bid: None,
        }]
    );

    Ok(())
}
