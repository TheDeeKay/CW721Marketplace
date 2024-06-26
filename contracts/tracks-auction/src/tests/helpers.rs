use crate::contract::instantiate;
use crate::execute::{bid, cancel_auction, receive_cw20, receive_nft, resolve_auction};
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::{
    to_json_binary, wasm_execute, Addr, BlockInfo, Coin, DepsMut, Env, Response, SubMsg,
};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ExecuteMsg::TransferNft;
use cw721::Cw721ReceiveMsg;
use cw_asset::Asset;
use cw_utils::Duration;
use tracks_auction_api::api::{AuctionId, PriceAssetUnchecked};
use tracks_auction_api::error::AuctionResult;
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;
use tracks_auction_api::msg::{Cw20HookMsg, InstantiateMsg};

pub const ADMIN: &str = "admin";

pub const USER1: &str = "user1";
pub const USER2: &str = "user2";
pub const USER3: &str = "user3";

pub const NFT_ADDR: &str = "nft_contract_addr";
pub const NFT_ADDR2: &str = "another_nft_contract_addr";

pub const CW20_ADDR: &str = "cw20_contract_addr";
pub const CW20_ADDR2: &str = "another_cw20_contract_addr";

pub const UANDR: &str = "uandr";
pub const UATOM: &str = "uatom";

pub const TOKEN1: &str = "1";

pub fn test_instantiate(
    deps: DepsMut,
    env: Env,
    instantiator: &str,
    whitelisted_nft: &str,
    price_asset: PriceAssetUnchecked,
) -> AuctionResult<Response> {
    instantiate(
        deps,
        env,
        mock_info(instantiator, &vec![]),
        InstantiateMsg {
            whitelisted_nft: whitelisted_nft.to_string(),
            price_asset,
        },
    )
}

pub fn instantiate_with_native_price_asset(
    deps: DepsMut,
    env: Env,
    instantiator: &str,
    whitelisted_nft: &str,
    native_denom: &str,
) -> AuctionResult<Response> {
    test_instantiate(
        deps,
        env,
        instantiator,
        whitelisted_nft,
        PriceAssetUnchecked::native(native_denom),
    )
}

pub fn instantiate_with_cw20_price_asset(
    deps: DepsMut,
    env: Env,
    instantiator: &str,
    whitelisted_nft: &str,
    cw20_addr: &str,
) -> AuctionResult<Response> {
    test_instantiate(
        deps,
        env,
        instantiator,
        whitelisted_nft,
        PriceAssetUnchecked::cw20(cw20_addr),
    )
}

pub fn create_test_auction(
    deps: DepsMut,
    env: Env,
    nft_contract: &str,
    token_id: &str,
    creator: &str,
    duration: Duration,
    minimum_bid_amount: u8,
    buyout_price: Option<u8>,
) -> AuctionResult<Response> {
    receive_nft(
        deps,
        env,
        mock_info(nft_contract, &vec![]),
        Cw721ReceiveMsg {
            sender: creator.to_string(),
            token_id: token_id.to_string(),
            msg: to_json_binary(&CreateAuction {
                duration,
                minimum_bid_amount: minimum_bid_amount.into(),
                buyout_price: buyout_price.map(|it| it.into()),
            })?,
        },
    )
}

pub fn test_bid(
    deps: DepsMut,
    env: Env,
    bidder: &str,
    auction_id: AuctionId,
    bid_amount: u8,
    bid_funds: &Vec<Coin>,
) -> AuctionResult<Response> {
    bid(
        deps,
        env,
        mock_info(bidder, bid_funds),
        auction_id,
        bid_amount.into(),
    )
}

pub fn test_cw20_bid(
    deps: DepsMut,
    env: Env,
    bidder: &str,
    auction_id: AuctionId,
    bid_amount: u8,
    amount_sent: u8,
    cw20: &str,
) -> AuctionResult<Response> {
    receive_cw20(
        deps,
        env,
        mock_info(cw20, &vec![]),
        Cw20ReceiveMsg {
            sender: bidder.to_string(),
            amount: amount_sent.into(),
            msg: to_json_binary(&Cw20HookMsg::Bid {
                auction_id,
                bid_amount: bid_amount.into(),
            })?,
        },
    )
}

pub fn test_resolve_auction(
    deps: DepsMut,
    env: Env,
    sender: &str,
    auction_id: AuctionId,
) -> AuctionResult<Response> {
    resolve_auction(deps, env, mock_info(sender, &vec![]), auction_id)
}

pub fn test_cancel_auction(
    deps: DepsMut,
    env: Env,
    sender: &str,
    auction_id: AuctionId,
) -> AuctionResult<Response> {
    cancel_auction(deps, env, mock_info(sender, &vec![]), auction_id)
}

pub fn no_funds() -> Vec<Coin> {
    vec![]
}

pub fn default_duration() -> Duration {
    Duration::Time(600)
}

pub fn after_height(env: &Env, height: u64) -> Env {
    Env {
        block: BlockInfo {
            height: env.block.height + height,
            ..env.block.clone()
        },
        ..env.clone()
    }
}

pub fn after_seconds(env: &Env, seconds: u64) -> Env {
    Env {
        block: BlockInfo {
            time: env.block.time.plus_seconds(seconds),
            ..env.block.clone()
        },
        ..env.clone()
    }
}

pub fn transfer_nft_msg(
    nft_contract: &str,
    recipient: &str,
    token_id: &str,
) -> anyhow::Result<SubMsg> {
    Ok(SubMsg::new(wasm_execute(
        nft_contract,
        &TransferNft {
            recipient: recipient.to_string(),
            token_id: token_id.to_string(),
        },
        vec![],
    )?))
}

pub fn transfer_native_funds(denom: &str, amount: u8, recipient: &str) -> anyhow::Result<SubMsg> {
    Ok(SubMsg::new(
        Asset::native(denom, amount).transfer_msg(recipient)?,
    ))
}

pub fn transfer_cw20_funds(cw20_addr: &str, amount: u8, recipient: &str) -> anyhow::Result<SubMsg> {
    Ok(SubMsg::new(
        Asset::cw20(Addr::unchecked(cw20_addr), amount).transfer_msg(recipient)?,
    ))
}
