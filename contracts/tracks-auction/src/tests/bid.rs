use crate::query::query_auction;
use crate::tests::helpers::{
    after_height, after_seconds, create_test_auction, default_duration,
    instantiate_with_native_price_asset, no_funds, test_bid, test_cw20_bid, ADMIN, CW20_ADDR,
    NFT_ADDR, TOKEN1, UANDR, UATOM, USER1, USER2, USER3,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, coin, coins, wasm_execute, Addr, BlockInfo, Env, SubMsg, Timestamp};
use cw721::Cw721ExecuteMsg::TransferNft;
use cw_asset::Asset;
use cw_utils::Duration::{Height, Time};
use tracks_auction_api::api::AuctionStatus::Resolved;
use tracks_auction_api::api::{Bid, PriceAsset};
use tracks_auction_api::error::AuctionError::{
    AuctionIdNotFound, BidLowerThanMinimum, BidWrongAsset, BiddingAfterAuctionEnded,
    InsufficientFundsForBid, NoBidFundsSupplied, UnnecessaryAssetsForBid,
};

#[test]
fn bid_with_no_asset_fails() -> anyhow::Result<()> {
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

    let result = test_bid(deps.as_mut(), env.clone(), USER2, 0, 0, &no_funds());

    assert_eq!(result, Err(NoBidFundsSupplied));

    Ok(())
}

#[test]
fn bid_on_non_existent_auction_fails() -> anyhow::Result<()> {
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

    let non_existent_auction_id = 2;
    let result = test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        non_existent_auction_id,
        5,
        &coins(5, UANDR),
    );

    assert_eq!(result, Err(AuctionIdNotFound));

    Ok(())
}

#[test]
fn bid_with_multiple_native_assets_fails() -> anyhow::Result<()> {
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

    let result = test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        5,
        &vec![coin(5, UANDR), coin(1, UATOM)],
    );

    assert_eq!(result, Err(UnnecessaryAssetsForBid));

    Ok(())
}

#[test]
fn bid_with_insufficient_funds_fails() -> anyhow::Result<()> {
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

    let bid_amount = 5;
    let funds_for_bid = 4;

    let result = test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        bid_amount,
        &coins(funds_for_bid, UANDR),
    );

    assert_eq!(result, Err(InsufficientFundsForBid));

    Ok(())
}

#[test]
fn bid_less_than_minimum_bid_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

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

    let result = test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        bid_amount,
        &coins(bid_amount.into(), UANDR),
    );

    assert_eq!(result, Err(BidLowerThanMinimum));

    Ok(())
}

#[test]
fn bid_wrong_asset_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let price_denom = UANDR;
    let bid_denom = UATOM;

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, price_denom)?;

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

    let result = test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        5,
        &coins(5, bid_denom),
    );

    assert_eq!(result, Err(BidWrongAsset));

    Ok(())
}

#[test]
fn bid_cw20_asset_fails() -> anyhow::Result<()> {
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

    let result = test_cw20_bid(deps.as_mut(), env.clone(), USER2, 0, 5, 5, CW20_ADDR);

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

    let response = test_bid(deps.as_mut(), env.clone(), USER2, 0, 5, &coins(5, UANDR))?;

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
            asset: PriceAsset::native(UANDR),
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

    test_bid(deps.as_mut(), env.clone(), USER2, 1, 5, &coins(5, UANDR))?;

    let auction = query_auction(deps.as_ref(), 1)?.auction;

    assert_eq!(
        auction.active_bid,
        Some(Bid {
            amount: 5u8.into(),
            asset: PriceAsset::native(UANDR),
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

    let first_bid_amount = 5;
    let second_bid_amount = first_bid_amount;

    test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        first_bid_amount,
        &coins(first_bid_amount.into(), UANDR),
    )?;

    let result = test_bid(
        deps.as_mut(),
        env.clone(),
        USER3,
        0,
        second_bid_amount,
        &coins(second_bid_amount.into(), UANDR),
    );

    assert_eq!(result, Err(BidLowerThanMinimum));

    Ok(())
}

#[test]
fn bid_over_existing_bid_replaces_and_refunds_existing_bid() -> anyhow::Result<()> {
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

    let first_bid_amount = 5;
    let second_bid_amount = 6;

    test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        first_bid_amount,
        &coins(first_bid_amount.into(), UANDR),
    )?;

    let new_block = BlockInfo {
        height: env.block.height + 10,
        time: Timestamp::from_seconds(env.block.time.seconds() + 120),
        chain_id: mock_env().block.chain_id,
    };

    let response = test_bid(
        deps.as_mut(),
        Env {
            block: new_block.clone(),
            ..env
        },
        USER3,
        0,
        second_bid_amount,
        &coins(second_bid_amount.into(), UANDR),
    )?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(
            Asset::native(UANDR, first_bid_amount).transfer_msg(USER2)?
        )],
    );

    let auction = query_auction(deps.as_ref(), 0)?.auction;

    assert_eq!(
        auction.active_bid,
        Some(Bid {
            amount: second_bid_amount.into(),
            asset: PriceAsset::native(UANDR),
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

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

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

    let result = test_bid(
        deps.as_mut(),
        after_seconds(&env, 54),
        USER2,
        0,
        5,
        &coins(5, UANDR),
    );

    assert_eq!(result, Err(BiddingAfterAuctionEnded));

    Ok(())
}

#[test]
fn bid_after_height_duration_auction_ended_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

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
        Some(200),
    )?;

    let result = test_bid(
        deps.as_mut(),
        after_height(&env, 29),
        USER2,
        0,
        5,
        &coins(5, UANDR),
    );

    assert_eq!(result, Err(BiddingAfterAuctionEnded));

    Ok(())
}

#[test]
fn bid_buyout_price_instantly_wins_the_auction() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

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

    let response = test_bid(
        deps.as_mut(),
        env.clone(),
        USER2,
        0,
        buyout_price,
        &coins(buyout_price.into(), UANDR),
    )?;

    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(Asset::native(UANDR, buyout_price).transfer_msg(USER1)?),
            SubMsg::new(wasm_execute(
                NFT_ADDR,
                &TransferNft {
                    recipient: USER2.to_string(),
                    token_id: TOKEN1.to_string()
                },
                vec![],
            )?,),
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
