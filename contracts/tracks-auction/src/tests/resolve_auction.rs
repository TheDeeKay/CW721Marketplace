use crate::contract::resolve_ended_auction;
use crate::tests::helpers::{
    create_test_auction, instantiate_with_native_price_asset, ADMIN, NFT_ADDR, TOKEN1, UANDR, USER1,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{wasm_execute, BlockInfo, Env, SubMsg};
use cw721::Cw721ExecuteMsg::TransferNft;
use cw_utils::Duration;
use cw_utils::Duration::Height;
use tracks_auction_api::error::AuctionError::{AuctionIdNotFound, AuctionStillInProgress};
use Duration::Time;

#[test]
fn resolve_ended_auction_that_does_not_exist_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    let result = resolve_ended_auction(deps.as_mut(), env.clone(), mock_info(USER1, &vec![]), 0);

    assert_eq!(result, Err(AuctionIdNotFound));

    Ok(())
}

#[test]
fn resolve_ended_time_duration_auction_that_did_not_expire_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        "1",
        USER1,
        Time(20),
        5,
    )?;

    let new_block = BlockInfo {
        time: env.block.time.plus_seconds(20),
        ..env.block.clone()
    };

    let result = resolve_ended_auction(
        deps.as_mut(),
        Env {
            block: new_block,
            ..env
        },
        mock_info(USER1, &vec![]),
        0,
    );

    assert_eq!(result, Err(AuctionStillInProgress));

    Ok(())
}

#[test]
fn resolve_ended_height_duration_auction_that_did_not_expire_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        "1",
        USER1,
        Height(15),
        5,
    )?;

    let new_block = BlockInfo {
        height: env.block.height + 15,
        ..env.block.clone()
    };

    let result = resolve_ended_auction(
        deps.as_mut(),
        Env {
            block: new_block,
            ..env
        },
        mock_info(USER1, &vec![]),
        0,
    );

    assert_eq!(result, Err(AuctionStillInProgress));

    Ok(())
}

#[test]
fn resolve_ended_height_duration_auction_with_no_active_bid_refunds_nft() -> anyhow::Result<()> {
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

    let new_block = BlockInfo {
        height: env.block.height + 16,
        ..env.block.clone()
    };

    let response = resolve_ended_auction(
        deps.as_mut(),
        Env {
            block: new_block,
            ..env
        },
        mock_info(USER1, &vec![]),
        0,
    )?;

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
fn resolve_ended_time_duration_auction_with_no_active_bid_refunds_nft() -> anyhow::Result<()> {
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

    let new_block = BlockInfo {
        time: env.block.time.plus_seconds(21),
        ..env.block.clone()
    };

    let response = resolve_ended_auction(
        deps.as_mut(),
        Env {
            block: new_block,
            ..env
        },
        mock_info(USER1, &vec![]),
        0,
    )?;

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
