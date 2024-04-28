use crate::tests::helpers::{
    create_test_auction, instantiate_with_native_price_asset, no_funds, test_bid, ADMIN, NFT_ADDR,
    TOKEN1, UANDR, USER1,
};
use cosmwasm_std::coins;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use tracks_auction_api::error::AuctionError::{AuctionIdNotFound, NoBidFundsSupplied};

#[test]
fn bid_with_no_asset_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let result = test_bid(deps.as_mut(), env.clone(), NFT_ADDR, 0, &no_funds());

    assert_eq!(result, Err(NoBidFundsSupplied));

    Ok(())
}

#[test]
fn bid_on_non_existent_auction_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let result = test_bid(deps.as_mut(), env.clone(), NFT_ADDR, 2, &coins(5, UANDR));

    assert_eq!(result, Err(AuctionIdNotFound));

    Ok(())
}

// TODO: consider cases such as sending multiple asset types for a bid, or sending more than attempting to bid
