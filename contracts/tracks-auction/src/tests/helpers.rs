use crate::contract::receive_nft;
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::{to_json_binary, DepsMut, Env, Response};
use cw721::Cw721ReceiveMsg;
use tracks_auction_api::error::AuctionResult;
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;

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
