use crate::execute::resolve_auction;
use crate::query::{query_auction, query_auctions};
use crate::tests::helpers::{
    after_height, after_seconds, create_test_auction, instantiate_with_native_price_asset,
    test_bid, test_cancel_auction, test_resolve_auction, transfer_nft_msg, ADMIN, NFT_ADDR, TOKEN1,
    UANDR, USER1, USER2,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{attr, coins, SubMsg};
use cw_asset::Asset;
use cw_utils::Duration;
use cw_utils::Duration::Height;
use tracks_auction_api::api::AuctionStatus::Resolved;
use tracks_auction_api::error::AuctionError::{
    AuctionCanceled, AuctionIdNotFound, AuctionResolved, AuctionStillInProgress,
};
use Duration::Time;

#[test]
fn resolve_auction_that_does_not_exist_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    let result = resolve_auction(deps.as_mut(), env.clone(), mock_info(USER1, &vec![]), 0);

    assert_eq!(result, Err(AuctionIdNotFound));

    Ok(())
}

#[test]
fn resolve_auction_time_duration_that_did_not_expire_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Time(20),
        5,
        None,
    )?;

    let result = test_resolve_auction(deps.as_mut(), after_seconds(&env, 20), USER1, 0);

    assert_eq!(result, Err(AuctionStillInProgress));

    Ok(())
}

#[test]
fn resolve_auction_height_duration_that_did_not_expire_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Height(15),
        5,
        None,
    )?;

    let result = test_resolve_auction(deps.as_mut(), after_height(&env, 15), USER1, 0);

    assert_eq!(result, Err(AuctionStillInProgress));

    Ok(())
}

#[test]
fn resolve_auction_height_duration_with_no_active_bid_refunds_nft() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Height(15),
        5,
        None,
    )?;

    let response = test_resolve_auction(deps.as_mut(), after_height(&env, 16), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![transfer_nft_msg(NFT_ADDR, USER1, TOKEN1)?]
    );

    assert_eq!(
        response.attributes,
        vec![attr("action", "resolve_auction"), attr("auction_id", "0"),],
    );

    Ok(())
}

#[test]
fn resolve_auction_ended_time_duration_with_no_active_bid_refunds_nft() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Time(20),
        5,
        Some(200),
    )?;

    let response = test_resolve_auction(deps.as_mut(), after_seconds(&env, 21), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![transfer_nft_msg(NFT_ADDR, USER1, TOKEN1)?]
    );

    Ok(())
}

#[test]
fn resolve_auction_with_active_bid_sends_nft_and_bid_to_new_owners() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Height(15),
        5,
        Some(7),
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 6, &coins(6, UANDR))?;

    let response = test_resolve_auction(deps.as_mut(), after_height(&env, 16), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![
            transfer_nft_msg(NFT_ADDR, USER2, TOKEN1)?,
            SubMsg::new(Asset::native(UANDR, 6u8).transfer_msg(USER1)?),
        ]
    );

    let auction = query_auction(deps.as_ref(), 0)?.auction;
    assert_eq!(auction.status, Resolved);

    assert!(query_auctions(deps.as_ref(), true, None, None)?
        .auctions
        .is_empty());
    assert_eq!(
        query_auctions(deps.as_ref(), false, None, None)?.auctions[0].status,
        Resolved
    );

    Ok(())
}

#[test]
fn resolve_auction_that_was_resolved_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Height(15),
        5,
        Some(200),
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 6, &coins(6, UANDR))?;

    let after_ending_env = after_height(&env, 16);

    test_resolve_auction(deps.as_mut(), after_ending_env.clone(), USER1, 0)?;

    let result = test_resolve_auction(deps.as_mut(), after_ending_env.clone(), USER1, 0);

    assert_eq!(result, Err(AuctionResolved));

    Ok(())
}

#[test]
fn resolve_auction_that_was_canceled_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Height(15),
        5,
        None,
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 6, &coins(6, UANDR))?;

    test_cancel_auction(deps.as_mut(), env.clone(), USER1, 0)?;

    let after_ending_env = after_height(&env, 16);

    let result = test_resolve_auction(deps.as_mut(), after_ending_env.clone(), USER1, 0);

    assert_eq!(result, Err(AuctionCanceled));

    Ok(())
}

#[test]
fn resolve_auction_that_was_instantly_bought_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Height(15),
        5,
        Some(20),
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 20, &coins(20, UANDR))?;

    let result = test_resolve_auction(deps.as_mut(), after_height(&env, 16), USER1, 0);

    assert_eq!(result, Err(AuctionResolved));

    Ok(())
}
