use crate::query::query_auction;
use crate::tests::helpers::{
    after_height, after_seconds, create_test_auction, default_duration,
    instantiate_with_cw20_price_asset, test_cw20_bid, ADMIN, CW20_ADDR, CW20_ADDR2, NFT_ADDR,
    TOKEN1, USER1, USER2, USER3,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, Addr, BlockInfo, Env, SubMsg, Timestamp};
use cw_asset::Asset;
use cw_utils::Duration::{Height, Time};
use tracks_auction_api::api::{Bid, PriceAsset};
use tracks_auction_api::error::AuctionError::{
    AuctionIdNotFound, BidLowerThanMinimum, BidWrongAsset, BiddingAfterAuctionEnded,
    InsufficientFundsForBid,
};

// TODO: those tests are essentially duplicates of native bids, abstract away to cut down on code duplication

// TODO: test what happens when sending denom to bid when price asset is cw20 (and vice versa)

#[test]
fn bid_on_non_existent_auction_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    let non_existent_auction_id = 2;
    let result = test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        non_existent_auction_id,
        5,
        5,
        CW20_ADDR,
    );

    assert_eq!(result, Err(AuctionIdNotFound));

    Ok(())
}

#[test]
fn bid_with_insufficient_funds_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    let bid_amount = 5;
    let funds_for_bid = 4;

    let result = test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        bid_amount,
        funds_for_bid,
        &CW20_ADDR,
    );

    assert_eq!(result, Err(InsufficientFundsForBid));

    Ok(())
}

#[test]
fn bid_less_than_minimum_bid_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    let minimum_bid_amount = 5;
    let bid_amount = 4;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        minimum_bid_amount,
    )?;

    let result = test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        bid_amount,
        bid_amount,
        CW20_ADDR,
    );

    assert_eq!(result, Err(BidLowerThanMinimum));

    Ok(())
}

#[test]
fn bid_wrong_asset_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let price_cw20 = CW20_ADDR;
    let bid_cw20 = CW20_ADDR2;

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, price_cw20)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    let result = test_cw20_bid(deps.as_mut(), env.clone(), USER2, 0, 5, 5, bid_cw20);

    assert_eq!(result, Err(BidWrongAsset));

    Ok(())
}

#[test]
fn bid_with_correct_funds_saves_it_as_active_bid() -> anyhow::Result<()> {
    let current_block = BlockInfo {
        height: 12345,
        time: Timestamp::from_seconds(23456),
        chain_id: mock_env().block.chain_id,
    };
    let mut deps = mock_dependencies();
    let env = Env {
        block: current_block.clone(),
        ..mock_env()
    };

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    let response = test_cw20_bid(deps.as_mut(), env.clone(), USER2, 0, 5, 5, CW20_ADDR)?;

    assert!(response.messages.is_empty());
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "bid"),
            attr("auction_id", "0"),
            attr("bid_amount", "5"),
        ]
    );

    let auction = query_auction(deps.as_ref(), 0)?.auction;

    assert_eq!(
        auction.active_bid,
        Some(Bid {
            amount: 5u8.into(),
            asset: PriceAsset::cw20(Addr::unchecked(CW20_ADDR)),
            bidder: Addr::unchecked(USER2),
            posted_at: current_block,
        })
    );

    Ok(())
}

#[test]
fn bid_on_second_auction_saves_it_as_active_bid_on_proper_auction() -> anyhow::Result<()> {
    let current_block = BlockInfo {
        height: 12345,
        time: Timestamp::from_seconds(23456),
        chain_id: mock_env().block.chain_id,
    };
    let mut deps = mock_dependencies();
    let env = Env {
        block: current_block.clone(),
        ..mock_env()
    };

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    test_cw20_bid(deps.as_mut(), env.clone(), USER2, 1, 5, 5, CW20_ADDR)?;

    let auction = query_auction(deps.as_ref(), 1)?.auction;

    assert_eq!(
        auction.active_bid,
        Some(Bid {
            amount: 5u8.into(),
            asset: PriceAsset::cw20(Addr::unchecked(CW20_ADDR)),
            bidder: Addr::unchecked(USER2),
            posted_at: current_block,
        })
    );

    let auction = query_auction(deps.as_ref(), 0)?.auction;
    assert!(auction.active_bid.is_none());

    Ok(())
}

#[test]
fn bid_after_existing_bid_at_current_amount_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    let first_bid_amount = 5;
    let second_bid_amount = first_bid_amount;

    test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        first_bid_amount,
        first_bid_amount,
        CW20_ADDR,
    )?;

    let result = test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER3,
        0,
        second_bid_amount,
        second_bid_amount,
        CW20_ADDR,
    );

    assert_eq!(result, Err(BidLowerThanMinimum));

    Ok(())
}

#[test]
fn bid_over_existing_bid_replaces_and_refunds_existing_bid() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    let first_bid_amount = 5;
    let second_bid_amount = 6;

    test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        first_bid_amount,
        first_bid_amount,
        CW20_ADDR,
    )?;

    let new_block = BlockInfo {
        height: env.block.height + 10,
        time: Timestamp::from_seconds(env.block.time.seconds() + 120),
        chain_id: mock_env().block.chain_id,
    };

    let response = test_cw20_bid(
        deps.as_mut(),
        Env {
            block: new_block.clone(),
            ..env
        },
        USER3,
        0,
        second_bid_amount,
        second_bid_amount,
        CW20_ADDR,
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(
            Asset::cw20(Addr::unchecked(CW20_ADDR), first_bid_amount).transfer_msg(USER2)?
        )],
    );

    let auction = query_auction(deps.as_ref(), 0)?.auction;

    assert_eq!(
        auction.active_bid,
        Some(Bid {
            amount: second_bid_amount.into(),
            asset: PriceAsset::cw20(Addr::unchecked(CW20_ADDR)),
            bidder: Addr::unchecked(USER3),
            posted_at: new_block,
        })
    );

    Ok(())
}

#[test]
fn bid_after_time_duration_auction_ended_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Time(53),
        5,
    )?;

    let result = test_cw20_bid(
        deps.as_mut(),
        after_seconds(&env, 54),
        USER2,
        0,
        5,
        5,
        CW20_ADDR,
    );

    assert_eq!(result, Err(BiddingAfterAuctionEnded));

    Ok(())
}

#[test]
fn bid_after_height_duration_auction_ended_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    let auction_created_at = BlockInfo {
        height: 1234,
        time: Timestamp::from_seconds(23456),
        chain_id: mock_env().block.chain_id,
    };

    create_test_auction(
        deps.as_mut(),
        Env {
            block: auction_created_at.clone(),
            ..env.clone()
        },
        NFT_ADDR,
        TOKEN1,
        USER1,
        Height(28),
        5,
    )?;

    let result = test_cw20_bid(
        deps.as_mut(),
        after_height(&env, 29),
        USER2,
        0,
        5,
        5,
        CW20_ADDR,
    );

    assert_eq!(result, Err(BiddingAfterAuctionEnded));

    Ok(())
}
