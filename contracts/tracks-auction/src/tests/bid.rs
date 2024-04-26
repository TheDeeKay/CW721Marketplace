use crate::contract::{bid, instantiate, receive_nft};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, to_json_binary};
use cw721::Cw721ReceiveMsg;
use tracks_auction_api::api::PriceAsset;
use tracks_auction_api::error::AuctionError::{AuctionIdNotFound, NoBidFundsSupplied};
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;
use tracks_auction_api::msg::InstantiateMsg;

// TODO: those consts repeat in every file, extract them somewhere (maybe even like a separate package because every package uses them)
const ADMIN: &str = "admin";

const USER1: &str = "user1";

const NFT_ADDR: &str = "nft_contract_addr";

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

    receive_nft(
        deps.as_mut(),
        env.clone(),
        mock_info(NFT_ADDR, &vec![]),
        Cw721ReceiveMsg {
            sender: USER1.to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&CreateAuction {
                minimum_bid_amount: 5u8.into(),
            })?,
        },
    )?;

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

    receive_nft(
        deps.as_mut(),
        env.clone(),
        mock_info(NFT_ADDR, &vec![]),
        Cw721ReceiveMsg {
            sender: USER1.to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&CreateAuction {
                minimum_bid_amount: 5u8.into(),
            })?,
        },
    )?;

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
