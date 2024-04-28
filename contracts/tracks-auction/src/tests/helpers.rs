use crate::contract::{bid, instantiate, receive_nft};
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::{to_json_binary, Coin, DepsMut, Env, Response};
use cw721::Cw721ReceiveMsg;
use tracks_auction_api::api::{AuctionId, PriceAsset};
use tracks_auction_api::error::AuctionResult;
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;
use tracks_auction_api::msg::InstantiateMsg;

// TODO: maybe extract to a separate test-general package because every package uses them
pub const ADMIN: &str = "admin";

pub const USER1: &str = "user1";

pub const NFT_ADDR: &str = "nft_contract_addr";
pub const NFT_ADDR2: &str = "another_nft_contract_addr";

pub const UANDR: &str = "uandr";
pub const UATOM: &str = "uatom";

pub const TOKEN1: &str = "1";

pub fn test_instantiate(
    deps: DepsMut,
    env: Env,
    instantiator: &str,
    whitelisted_nft: &str,
    price_asset: PriceAsset,
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
        PriceAsset::native(native_denom),
    )
}

pub fn create_test_auction(
    deps: DepsMut,
    env: Env,
    nft_contract: &str,
    token_id: &str,
    creator: &str,
    minimum_bid_amount: u8,
) -> AuctionResult<Response> {
    receive_nft(
        deps,
        env,
        mock_info(nft_contract, &vec![]),
        Cw721ReceiveMsg {
            sender: creator.to_string(),
            token_id: token_id.to_string(),
            msg: to_json_binary(&CreateAuction {
                minimum_bid_amount: minimum_bid_amount.into(),
            })?,
        },
    )
}

pub fn test_bid(
    deps: DepsMut,
    env: Env,
    nft_contract: &str,
    auction_id: AuctionId,
    bid_funds: &Vec<Coin>,
) -> AuctionResult<Response> {
    bid(deps, env, mock_info(nft_contract, bid_funds), auction_id)
}

pub fn no_funds() -> Vec<Coin> {
    vec![]
}