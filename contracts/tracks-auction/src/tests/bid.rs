use crate::query::query_auction;
use crate::tests::helpers::{
    create_test_auction, instantiate_with_native_price_asset, no_funds, test_bid, ADMIN, NFT_ADDR,
    TOKEN1, UANDR, UATOM, USER1, USER2, USER3,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{coin, coins, Addr, BankMsg, SubMsg};
use tracks_auction_api::api::{Bid, PriceAsset};
use tracks_auction_api::error::AuctionError::{
    AuctionIdNotFound, BidLowerThanMinimum, BidWrongAsset, InsufficientFundsForBid,
    NoBidFundsSupplied, UnnecessaryAssetsForBid,
};

// TODO: go through expected errors and modify them to (usually) be BidBelowMinimum or similar, we can consolidate the myriad types of errors used

#[test]
fn bid_with_no_asset_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let result = test_bid(deps.as_mut(), env.clone(), USER2, 0, 0, &no_funds());

    assert_eq!(result, Err(NoBidFundsSupplied));

    Ok(())
}

#[test]
fn bid_on_non_existent_auction_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

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

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

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

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

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
        minimum_bid_amount,
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

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

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
fn bid_with_correct_funds_saves_it_as_active_bid() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let response = test_bid(deps.as_mut(), env.clone(), USER2, 0, 5, &coins(5, UANDR))?;

    assert!(response.messages.is_empty());

    let auction = query_auction(deps.as_ref(), 0)?.auction;

    assert_eq!(
        auction.active_bid,
        Some(Bid {
            amount: 5u8.into(),
            asset: PriceAsset::native(UANDR),
            bidder: Addr::unchecked(USER2),
        })
    );

    Ok(())
}

#[test]
fn bid_after_existing_bid_at_current_amount_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

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

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

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

    let response = test_bid(
        deps.as_mut(),
        env.clone(),
        USER3,
        0,
        second_bid_amount,
        &coins(second_bid_amount.into(), UANDR),
    )?;

    assert!(response.messages.contains(&SubMsg::new(BankMsg::Send {
        to_address: USER2.to_string(),
        amount: coins(first_bid_amount.into(), UANDR)
    })));

    let auction = query_auction(deps.as_ref(), 0)?.auction;

    assert_eq!(
        auction.active_bid,
        Some(Bid {
            amount: second_bid_amount.into(),
            asset: PriceAsset::native(UANDR),
            bidder: Addr::unchecked(USER3),
        })
    );

    Ok(())
}
