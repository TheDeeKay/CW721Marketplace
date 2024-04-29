use crate::cw721_tracks::cw721_tracks_helpers::{
    mint_nft, query_nft, store_and_instantiate_cw721_tracks,
};
use crate::helpers::{UATOM, USER1};
use crate::tracks_auction::tracks_auction_helpers::store_and_instantiate_tracks_auction;
use cosmwasm_std::to_json_binary;
use cw721::Cw721ExecuteMsg::SendNft;
use cw721_tracks_api::api::TrackMetadata;
use cw_multi_test::{App, Executor, IntoAddr};
use cw_utils::Duration;
use tracks_auction_api::api::{
    AuctionStatus, AuctionsResponse, PriceAsset, PriceAssetUnchecked, TrackAuction,
};
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;
use tracks_auction_api::msg::QueryMsg::Auctions;
use AuctionStatus::Active;

#[test]
fn nft_minting() -> anyhow::Result<()> {
    let mut app = App::default();

    let (_, cw721_tracks) = store_and_instantiate_cw721_tracks(&mut app)?;

    store_and_instantiate_tracks_auction(
        &mut app,
        cw721_tracks.to_string(),
        PriceAssetUnchecked::native(UATOM),
    )?;

    let track_metadata = TrackMetadata {
        artist_name: "Boden".to_string(),
        album: None,
        track_name: "Debt Spiral".to_string(),
        audio_track_url: "https://www.usdebtclock.org/".to_string(),
    };

    mint_nft(
        &mut app,
        cw721_tracks.clone(),
        USER1,
        "1",
        None,
        track_metadata.clone(),
    )?;

    let nft = query_nft(&mut app, cw721_tracks.clone(), "1")?;

    assert_eq!(nft.token_uri, None);

    assert_eq!(nft.extension, track_metadata);

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
        PriceAssetUnchecked::native(UATOM),
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
                buyout_price: Some(200u8.into()),
            })?,
        },
        &vec![],
    )?;

    let response: AuctionsResponse = app.wrap().query_wasm_smart(
        tracks_auction.clone(),
        &Auctions {
            active_auctions: true,
            start_after: None,
            limit: None,
        },
    )?;

    assert_eq!(
        response.auctions,
        vec![TrackAuction {
            status: Active,
            created_at: app.block_info(),
            duration,
            id: 0,
            creator: USER1.into_addr(),
            nft_contract: cw721_tracks,
            track_token_id: track_token_id.to_string(),
            minimum_bid_amount: 4u128.into(),
            price_asset: PriceAsset::native(UATOM),
            active_bid: None,
            buyout_price: Some(200u8.into()),
        }]
    );

    Ok(())
}
