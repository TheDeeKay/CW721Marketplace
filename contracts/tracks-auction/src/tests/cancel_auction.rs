use crate::query::{query_auction, query_auctions};
use crate::tests::helpers::{
    after_seconds, create_test_auction, default_duration, instantiate_with_native_price_asset,
    test_bid, test_cancel_auction, test_resolve_auction, transfer_nft_msg, ADMIN, NFT_ADDR, TOKEN1,
    UANDR, USER1, USER2,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, coins, SubMsg};
use cw_asset::Asset;
use cw_utils::Duration::Time;
use tracks_auction_api::api::AuctionStatus::Canceled;
use tracks_auction_api::error::AuctionError::{
    AuctionCanceled, AuctionExpired, AuctionResolved, Unauthorized,
};

#[test]
fn cancel_auction_only_permitted_to_auction_creator() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
        None,
    )?;

    let result = test_cancel_auction(deps.as_mut(), env, USER2, 0);

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn cancel_auction_on_expired_auction_fails() -> anyhow::Result<()> {
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

    let result = test_cancel_auction(deps.as_mut(), after_seconds(&env, 21), USER1, 0);

    assert_eq!(result, Err(AuctionExpired));

    Ok(())
}

#[test]
fn cancel_auction_on_resolved_auction_fails() -> anyhow::Result<()> {
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

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 5, &coins(5, UANDR))?;

    let env_after_expiration = after_seconds(&env, 21);

    test_resolve_auction(deps.as_mut(), env_after_expiration.clone(), ADMIN, 0)?;

    let result = test_cancel_auction(deps.as_mut(), env_after_expiration, USER1, 0);

    assert_eq!(result, Err(AuctionResolved));

    Ok(())
}

#[test]
fn cancel_auction_on_instantly_bought_auction_fails() -> anyhow::Result<()> {
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
        Some(20),
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 20, &coins(20, UANDR))?;

    let result = test_cancel_auction(deps.as_mut(), env, USER1, 0);

    assert_eq!(result, Err(AuctionResolved));

    Ok(())
}

#[test]
fn cancel_auction_on_canceled_auction_fails() -> anyhow::Result<()> {
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

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 5, &coins(5, UANDR))?;

    test_cancel_auction(deps.as_mut(), env.clone(), USER1, 0)?;

    let result = test_cancel_auction(deps.as_mut(), env, USER1, 0);

    assert_eq!(result, Err(AuctionCanceled));

    Ok(())
}

#[test]
fn cancel_auction_with_no_bids_sends_back_nft_to_creator() -> anyhow::Result<()> {
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

    let response = test_cancel_auction(deps.as_mut(), env.clone(), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![transfer_nft_msg(NFT_ADDR, USER1, TOKEN1)?],
    );

    assert_eq!(
        response.attributes,
        vec![attr("action", "cancel_auction"), attr("auction_id", "0"),],
    );

    assert_eq!(query_auction(deps.as_ref(), 0)?.auction.status, Canceled);

    assert!(query_auctions(deps.as_ref(), true, None, None)?
        .auctions
        .is_empty());
    assert_eq!(
        query_auctions(deps.as_ref(), false, None, None)?.auctions[0].status,
        Canceled
    );

    Ok(())
}

#[test]
fn cancel_auction_with_bid_sends_back_nft_to_creator_and_bid_to_bidder() -> anyhow::Result<()> {
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

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 5, &coins(5, UANDR))?;
    test_bid(deps.as_mut(), env.clone(), ADMIN, 0, 8, &coins(8, UANDR))?;

    let response = test_cancel_auction(deps.as_mut(), env.clone(), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![
            transfer_nft_msg(NFT_ADDR, USER1, TOKEN1)?,
            SubMsg::new(Asset::native(UANDR, 8u8).transfer_msg(ADMIN)?)
        ]
    );

    Ok(())
}
