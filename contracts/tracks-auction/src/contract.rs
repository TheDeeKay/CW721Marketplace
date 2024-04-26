use crate::auctions::{load_auctions, save_auction};
use crate::state::{add_whitelisted_cw721, is_cw721_whitelisted};
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError,
};
use cw721::Cw721ReceiveMsg;
use tracks_auction_api::api::{AuctionsResponse, TrackAuction};
use tracks_auction_api::error::AuctionError::Cw721NotWhitelisted;
use tracks_auction_api::error::{AuctionError, AuctionResult};
use tracks_auction_api::msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use Cw721HookMsg::CreateAuction;
use ExecuteMsg::ReceiveNft;

// Version info for migration
const CONTRACT_NAME: &str = "tracks-auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> AuctionResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    for cw721 in _msg.whitelisted_nfts {
        let addr = deps.api.addr_validate(&cw721)?;
        add_whitelisted_cw721(deps.storage, addr)?;
    }

    Ok(Response::new()) // TODO: add some attributes
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> AuctionResult<Response> {
    match msg {
        ReceiveNft(nft_msg) => receive_nft(deps, env, info, nft_msg),
    }
}

// TODO: move to another file
fn receive_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> AuctionResult<Response> {
    // only whitelisted NFTs can call this
    if !is_cw721_whitelisted(deps.storage, info.sender.clone())? {
        return Err(Cw721NotWhitelisted);
    }

    match from_json(msg.msg) {
        Ok(CreateAuction { minimum_bid_amount }) => {
            save_auction(
                deps.storage,
                TrackAuction {
                    track_nft_contract: info.sender,
                    track_token_id: msg.token_id,
                    minimum_bid_amount,
                },
            )?;
            Ok(Response::new()) // TODO: add attributes
        }
        _ => Err(StdError::generic_err("unknown NFT receive hook message").into()),
    }
}

// TODO: move to another file? or just the individual parts
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Binary, AuctionError> {
    // TODO: match the msg
    Ok(to_json_binary(&AuctionsResponse {
        auctions: load_auctions(deps.storage)?,
    })?)
}
