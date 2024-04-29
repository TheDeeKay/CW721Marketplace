use crate::query::query_auction;
use crate::tests::helpers::{
    after_height, after_seconds, create_test_auction, default_duration,
    instantiate_with_cw20_price_asset, test_bid, test_cw20_bid, transfer_cw20_funds,
    transfer_nft_msg, ADMIN, CW20_ADDR, CW20_ADDR2, NFT_ADDR, TOKEN1, UANDR, USER1, USER2, USER3,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, coins, Addr, BlockInfo, Env, Timestamp};
use cw_utils::Duration::{Height, Time};
use tracks_auction_api::api::AuctionStatus::Resolved;
use tracks_auction_api::api::{Bid, PriceAsset};
use tracks_auction_api::error::AuctionError::{
    AuctionIdNotFound, BidLowerThanMinimum, BidWrongAsset, BiddingAfterAuctionEnded,
    InsufficientFundsForBid,
};

// TODO: those tests are essentially duplicates of native bids, abstract away to cut down on code duplication

#[test]
fn bid_cw20_on_non_existent_auction_fails() -> anyhow::Result<()> {
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
        None,
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
fn bid_cw20_with_insufficient_funds_fails() -> anyhow::Result<()> {
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
        None,
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
fn bid_cw20_less_than_minimum_bid_fails() -> anyhow::Result<()> {
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
        None,
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
fn bid_cw20_wrong_asset_fails() -> anyhow::Result<()> {
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
        None,
    )?;

    let result = test_cw20_bid(deps.as_mut(), env.clone(), USER2, 0, 5, 5, bid_cw20);

    assert_eq!(result, Err(BidWrongAsset));

    Ok(())
}

#[test]
fn bid_cw20_native_asset_fails() -> anyhow::Result<()> {
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
        None,
    )?;

    let result = test_bid(deps.as_mut(), env.clone(), USER2, 0, 5, &coins(5, UANDR));

    assert_eq!(result, Err(BidWrongAsset));

    Ok(())
}

#[test]
fn bid_cw20_with_correct_funds_saves_it_as_active_bid() -> anyhow::Result<()> {
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
        Some(200),
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
fn bid_cw20_on_second_auction_saves_it_as_active_bid_on_proper_auction() -> anyhow::Result<()> {
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
        None,
    )?;

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
fn bid_cw20_after_existing_bid_at_current_amount_fails() -> anyhow::Result<()> {
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
        Some(200),
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
fn bid_cw20_over_existing_bid_replaces_and_refunds_existing_bid() -> anyhow::Result<()> {
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
        None,
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
        vec![transfer_cw20_funds(CW20_ADDR, first_bid_amount, USER2)?],
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
fn bid_cw20_after_time_duration_auction_ended_fails() -> anyhow::Result<()> {
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
        None,
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
fn bid_cw20_after_height_duration_auction_ended_fails() -> anyhow::Result<()> {
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
        None,
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

#[test]
fn bid_cw20_buyout_price_instantly_wins_the_auction() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    let buyout_price = 10;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
        Some(buyout_price),
    )?;

    let response = test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        buyout_price,
        buyout_price,
        CW20_ADDR,
    )?;

    assert_eq!(
        response.messages,
        vec![
            transfer_cw20_funds(CW20_ADDR, buyout_price, USER1)?,
            transfer_nft_msg(NFT_ADDR, USER2, TOKEN1)?,
        ],
    );

    assert_eq!(
        response.attributes,
        vec![
            attr("action", "instant_buyout"),
            attr("auction_id", "0"),
            attr("bid_amount", buyout_price.to_string())
        ],
    );

    let auction = query_auction(deps.as_ref(), 0)?.auction;

    assert_eq!(auction.status, Resolved);

    Ok(())
}

#[test]
fn bid_cw20_buyout_price_refunds_previous_bid() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_cw20_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, CW20_ADDR)?;

    let buyout_price = 10;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
        Some(buyout_price),
    )?;

    test_cw20_bid(deps.as_mut(), env.clone(), USER2, 0, 6, 6, CW20_ADDR)?;

    let response = test_cw20_bid(
        deps.as_mut(),
        env.clone(),
        USER3,
        0,
        buyout_price,
        buyout_price,
        CW20_ADDR,
    )?;

    assert!(response
        .messages
        .contains(&transfer_cw20_funds(CW20_ADDR, 6, USER2)?));

    Ok(())
}
