use crate::contract::{execute, instantiate};
use crate::query::query_auctions;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_json_binary, Addr, Uint128};
use cw721::Cw721ReceiveMsg;
use tracks_auction_api::api::TrackAuction;
use tracks_auction_api::error::AuctionError::Cw721NotWhitelisted;
use tracks_auction_api::msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg};
use Cw721HookMsg::CreateAuction;
use ExecuteMsg::ReceiveNft;

// TODO: those consts repeat in every file, extract them somewhere (maybe even like a separate package because every package uses them)
const ADMIN: &str = "admin";

const USER1: &str = "user1";

const NFT_ADDR: &str = "nft_contract_addr";
const NFT_ADDR2: &str = "another_nft_contract_addr";

#[test]
fn create_auction_for_non_whitelisted_nft_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        InstantiateMsg {
            whitelisted_nft: NFT_ADDR.to_string(),
        },
    )?;

    let result = execute(
        deps.as_mut(),
        env,
        mock_info(NFT_ADDR2, &vec![]),
        ReceiveNft(Cw721ReceiveMsg {
            sender: ADMIN.to_string(),
            token_id: "1".to_string(),
            msg: to_json_binary(&CreateAuction {
                minimum_bid_amount: Uint128::zero(),
            })?,
        }),
    );

    assert_eq!(result, Err(Cw721NotWhitelisted));

    Ok(())
}

#[test]
fn create_auction_saves_it_with_relevant_data() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        InstantiateMsg {
            whitelisted_nft: NFT_ADDR.to_string(),
        },
    )?;

    let track_token_id = "first_track";

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(NFT_ADDR, &vec![]),
        ReceiveNft(Cw721ReceiveMsg {
            sender: USER1.to_string(),
            token_id: track_token_id.to_string(),
            msg: to_json_binary(&CreateAuction {
                minimum_bid_amount: 4u128.into(),
            })?,
        }),
    )?;

    let response = query_auctions(deps.as_ref())?;

    assert_eq!(
        response.auctions,
        vec![TrackAuction {
            submitter: Addr::unchecked(USER1),
            track_token_id: track_token_id.to_string(),
            minimum_bid_amount: 4u128.into(),
        }]
    );

    Ok(())
}
