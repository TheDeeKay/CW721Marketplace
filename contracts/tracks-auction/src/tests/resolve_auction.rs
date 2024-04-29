use crate::contract::resolve_auction;
use crate::query::query_auction;
use crate::tests::helpers::{
    after_height, after_seconds, create_test_auction, instantiate_with_native_price_asset,
    test_bid, test_resolve_auction, ADMIN, NFT_ADDR, TOKEN1, UANDR, USER1, USER2,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, wasm_execute, SubMsg};
use cw721::Cw721ExecuteMsg::TransferNft;
use cw_asset::Asset;
use cw_utils::Duration;
use cw_utils::Duration::Height;
use tracks_auction_api::api::AuctionStatus::Resolved;
use tracks_auction_api::error::AuctionError::{
    AuctionAlreadyResolved, AuctionIdNotFound, AuctionStillInProgress,
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
    )?;

    let response = test_resolve_auction(deps.as_mut(), after_height(&env, 16), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(wasm_execute(
            NFT_ADDR,
            &TransferNft {
                recipient: USER1.to_string(),
                token_id: TOKEN1.to_string()
            },
            vec![],
        )?)]
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
    )?;

    let response = test_resolve_auction(deps.as_mut(), after_seconds(&env, 21), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![SubMsg::new(wasm_execute(
            NFT_ADDR,
            &TransferNft {
                recipient: USER1.to_string(),
                token_id: TOKEN1.to_string()
            },
            vec![],
        )?)]
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
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 6, &coins(6, UANDR))?;

    let response = test_resolve_auction(deps.as_mut(), after_height(&env, 16), USER1, 0)?;

    assert_eq!(
        response.messages,
        vec![
            SubMsg::new(wasm_execute(
                NFT_ADDR,
                &TransferNft {
                    recipient: USER2.to_string(),
                    token_id: TOKEN1.to_string()
                },
                vec![],
            )?),
            SubMsg::new(Asset::native(UANDR, 6u8).transfer_msg(USER1)?),
        ]
    );

    let auction = query_auction(deps.as_ref(), 0)?.auction;
    assert_eq!(auction.status, Resolved);

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
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 6, &coins(6, UANDR))?;

    let after_ending_env = after_height(&env, 16);

    test_resolve_auction(deps.as_mut(), after_ending_env.clone(), USER1, 0)?;

    let result = test_resolve_auction(deps.as_mut(), after_ending_env.clone(), USER1, 0);

    assert_eq!(result, Err(AuctionAlreadyResolved));

    Ok(())
}

// TODO: resolve cancelled auction fails
// TODO: resolve instantly-bought auction fails
