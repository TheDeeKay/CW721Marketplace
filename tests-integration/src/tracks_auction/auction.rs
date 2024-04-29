use crate::cw721_tracks::cw721_tracks_helpers::{
    mint_nft, store_and_instantiate_cw721_tracks, NftQueries,
};
use crate::helpers::{BalanceQuery, MoveBlock, ADMIN, UATOM, USER1, USER2, USER3};
use crate::tracks_auction::tracks_auction_helpers::{
    assert_active_bid, bid_on_auction, create_nft_auction, resolve_auction,
    store_and_instantiate_tracks_auction,
};
use cosmwasm_std::{coin, coins, to_json_binary};
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
use Duration::Time;

#[test]
fn single_nft_auction_with_bidders() -> anyhow::Result<()> {
    let mut app = App::default();

    let (_, cw721_tracks) = store_and_instantiate_cw721_tracks(&mut app)?;

    let (_, tracks_auction) = store_and_instantiate_tracks_auction(
        &mut app,
        cw721_tracks.to_string(),
        PriceAssetUnchecked::native(UATOM),
    )?;

    let token_id = "tokenID";

    mint_nft(
        &mut app,
        cw721_tracks.clone(),
        USER1,
        token_id,
        None,
        TrackMetadata {
            artist_name: "Boden".to_string(),
            album: None,
            track_name: "Debt Spiral".to_string(),
            audio_track_url: "https://www.usdebtclock.org/".to_string(),
        }
        .clone(),
    )?;

    create_nft_auction(
        &mut app,
        cw721_tracks.clone(),
        tracks_auction.clone(),
        USER1,
        token_id,
        Time(100),
        100,
        None,
    )?;

    // lower than minimum bid fails
    let result = bid_on_auction(&mut app, tracks_auction.clone(), USER2, 0, coin(99, UATOM));
    assert!(result.is_err());

    // first over the minimum bid
    bid_on_auction(&mut app, tracks_auction.clone(), USER2, 0, coin(100, UATOM))?;
    assert_active_bid(
        &app,
        tracks_auction.clone(),
        0,
        USER2,
        coin(100, UATOM),
        app.block_info(),
    );

    app.move_time_sec(20);

    // resolving before auction is over fails
    let result = resolve_auction(&mut app, tracks_auction.clone(), ADMIN, 0);
    assert!(result.is_err());

    // next higher bid becomes the new active bid and refunds previous bid
    bid_on_auction(&mut app, tracks_auction.clone(), USER3, 0, coin(101, UATOM))?;
    assert_active_bid(
        &app,
        tracks_auction.clone(),
        0,
        USER3,
        coin(101, UATOM),
        app.block_info(),
    );
    app.assert_balance(USER2, coins(100, UATOM));

    // next bid must be higher than the current
    let res = bid_on_auction(&mut app, tracks_auction.clone(), USER2, 0, coin(101, UATOM));
    assert!(res.is_err());

    // auction expires
    app.move_time_sec(81);

    // after auction expires, no bids are accepted
    let res = bid_on_auction(&mut app, tracks_auction.clone(), USER2, 0, coin(102, UATOM));
    assert!(res.is_err());

    // resolving auction sends bid to auction owner, NFT to auction winner
    resolve_auction(&mut app, tracks_auction.clone(), ADMIN, 0)?;
    app.assert_nft_owner(cw721_tracks.clone(), token_id, USER3);
    app.assert_balance(USER1, coins(101, UATOM));

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

    let duration = Time(200);

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
