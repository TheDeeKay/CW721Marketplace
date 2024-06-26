use crate::query::{query_auction, query_auctions};
use crate::tests::helpers::{
    create_test_auction, default_duration, instantiate_with_native_price_asset, ADMIN, NFT_ADDR,
    NFT_ADDR2, TOKEN1, UANDR, UATOM, USER1,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, Addr, BlockInfo, Env, Timestamp};
use cw_utils::Duration;
use cw_utils::Duration::Height;
use tracks_auction_api::api::AuctionStatus::Active;
use tracks_auction_api::api::{PriceAsset, TrackAuction};
use tracks_auction_api::error::AuctionError::{Cw721NotWhitelisted, InvalidAuctionDuration};
use Duration::Time;

#[test]
fn create_auction_for_non_whitelisted_nft_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    let result = create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR2,
        TOKEN1,
        ADMIN,
        default_duration(),
        0,
        None,
    );

    assert_eq!(result, Err(Cw721NotWhitelisted));

    Ok(())
}

#[test]
fn create_auction_with_time_0_duration_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    let result = create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        ADMIN,
        Time(0),
        0,
        Some(200),
    );

    assert_eq!(result, Err(InvalidAuctionDuration));

    Ok(())
}

#[test]
fn create_auction_with_height_0_duration_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    let result = create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        ADMIN,
        Height(0),
        0,
        None,
    );

    assert_eq!(result, Err(InvalidAuctionDuration));

    Ok(())
}

#[test]
fn create_auction_saves_it_with_relevant_data() -> anyhow::Result<()> {
    let current_block = BlockInfo {
        height: 55214,
        time: Timestamp::from_nanos(5521400000),
        chain_id: mock_env().block.chain_id,
    };

    let mut deps = mock_dependencies();
    let env = Env {
        block: current_block.clone(),
        ..mock_env()
    };

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UATOM)?;

    let track_token_id = "first_track";
    let duration = Time(24);

    let response = create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        track_token_id,
        USER1,
        duration.clone(),
        4,
        Some(213),
    )?;

    assert_eq!(
        response.attributes,
        vec![attr("action", "create_auction"), attr("auction_id", "0"),],
    );

    let expected_auction = TrackAuction {
        status: Active,
        created_at: current_block,
        duration,
        id: 0,
        creator: Addr::unchecked(USER1),
        nft_contract: Addr::unchecked(NFT_ADDR),
        track_token_id: track_token_id.to_string(),
        minimum_bid_amount: 4u8.into(),
        price_asset: PriceAsset::native("uatom"),
        active_bid: None,
        buyout_price: Some(213u8.into()),
    };

    let response = query_auction(deps.as_ref(), 0)?;

    assert_eq!(response.auction, expected_auction.clone());

    assert_eq!(
        query_auctions(deps.as_ref(), true, None, None)?.auctions,
        vec![expected_auction]
    );

    assert!(query_auctions(deps.as_ref(), false, None, None)?
        .auctions
        .is_empty());

    Ok(())
}

#[test]
fn create_auction_increments_auction_ids() -> anyhow::Result<()> {
    let current_block = BlockInfo {
        height: 55214,
        time: Timestamp::from_nanos(5521400000),
        chain_id: mock_env().block.chain_id,
    };

    let mut deps = mock_dependencies();
    let env = Env {
        block: current_block.clone(),
        ..mock_env()
    };

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UATOM)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        "5",
        USER1,
        default_duration(),
        4,
        Some(200),
    )?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        "6",
        USER1,
        default_duration(),
        4,
        None,
    )?;

    assert_eq!(query_auction(deps.as_ref(), 0)?.auction.id, 0);

    assert_eq!(query_auction(deps.as_ref(), 1)?.auction.id, 1);

    let response = query_auctions(deps.as_ref(), true, None, None)?;

    assert_eq!(response.auctions[0].id, 0);
    assert_eq!(response.auctions[1].id, 1);

    // ensure pagination works for query_auctions

    let response = query_auctions(deps.as_ref(), true, None, Some(1))?;
    assert_eq!(response.auctions.len(), 1);
    assert_eq!(response.auctions[0].id, 0);

    let response = query_auctions(deps.as_ref(), true, Some(0), None)?;
    assert_eq!(response.auctions.len(), 1);
    assert_eq!(response.auctions[0].id, 1);

    assert!(query_auctions(deps.as_ref(), false, None, None)?
        .auctions
        .is_empty());

    Ok(())
}
