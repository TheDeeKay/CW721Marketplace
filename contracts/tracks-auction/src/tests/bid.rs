use crate::tests::helpers::{
    create_test_auction, instantiate_with_native_price_asset, no_funds, test_bid, ADMIN, NFT_ADDR,
    TOKEN1, UANDR, UATOM, USER1,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{coin, coins};
use tracks_auction_api::error::AuctionError::{
    AuctionIdNotFound, InsufficientFundsForBid, NoBidFundsSupplied, UnnecessaryAssetsForBid,
};

// TODO: go through expected errors and modify them to (usually) be BidBelowMinimum or similar

#[test]
fn bid_with_no_asset_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let result = test_bid(deps.as_mut(), env.clone(), NFT_ADDR, 0, 0, &no_funds());

    assert_eq!(result, Err(NoBidFundsSupplied));

    Ok(())
}

#[test]
fn bid_on_non_existent_auction_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let result = test_bid(deps.as_mut(), env.clone(), NFT_ADDR, 2, 5, &coins(5, UANDR));

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
        NFT_ADDR,
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

    let result = test_bid(deps.as_mut(), env.clone(), NFT_ADDR, 0, 5, &coins(4, UANDR));

    assert_eq!(result, Err(InsufficientFundsForBid));

    Ok(())
}

// TODO: next is bid under minimum
