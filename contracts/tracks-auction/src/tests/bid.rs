use crate::contract::{bid, instantiate};
use crate::tests::helpers::{create_test_auction, ADMIN, NFT_ADDR, TOKEN1, USER1};
use cosmwasm_std::coins;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use tracks_auction_api::api::PriceAsset;
use tracks_auction_api::error::AuctionError::{AuctionIdNotFound, NoBidFundsSupplied};
use tracks_auction_api::msg::InstantiateMsg;

#[test]
fn bid_with_no_asset_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        InstantiateMsg {
            whitelisted_nft: NFT_ADDR.to_string(),
            price_asset: PriceAsset::native("uandr"),
        },
    )?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let result = bid(deps.as_mut(), env.clone(), mock_info(NFT_ADDR, &vec![]), 0);

    assert_eq!(result, Err(NoBidFundsSupplied));

    Ok(())
}

#[test]
fn bid_on_non_existent_auction_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        InstantiateMsg {
            whitelisted_nft: NFT_ADDR.to_string(),
            price_asset: PriceAsset::native("uandr"),
        },
    )?;

    create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR, TOKEN1, USER1, 5)?;

    let result = bid(
        deps.as_mut(),
        env.clone(),
        mock_info(NFT_ADDR, &coins(5, "uandr")),
        2,
    );

    assert_eq!(result, Err(AuctionIdNotFound));

    Ok(())
}

// TODO: consider cases such as sending multiple asset types for a bid, or sending more than attempting to bid
