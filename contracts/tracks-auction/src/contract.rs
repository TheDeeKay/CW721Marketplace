use crate::config::save_config;
use crate::execute;
use crate::query::{query_auction, query_auctions, query_config};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};
use tracks_auction_api::api::Config;
use tracks_auction_api::error::{AuctionError, AuctionResult};
use tracks_auction_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use ExecuteMsg::{CancelAuction, ReceiveNft, ResolveAuction};
use QueryMsg::{Auction, Auctions};

// Version info for migration
const CONTRACT_NAME: &str = "tracks-auction";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> AuctionResult<Response> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let nft_addr = deps.api.addr_validate(&msg.whitelisted_nft)?;

    let config = Config {
        whitelisted_nft: nft_addr,
        price_asset: msg.price_asset,
    };
    save_config(deps.storage, &config)?;

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
        ReceiveNft(nft_msg) => execute::receive_nft(deps, env, info, nft_msg),
        ExecuteMsg::Bid {
            auction_id,
            bid_amount,
        } => execute::bid(deps, env, info, auction_id, bid_amount),
        ResolveAuction { auction_id } => execute::resolve_auction(deps, env, info, auction_id),
        CancelAuction { auction_id } => execute::cancel_auction(deps, env, info, auction_id),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, AuctionError> {
    let response = match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?)?,
        Auction { id } => to_json_binary(&query_auction(deps, id)?)?,
        Auctions { start_after, limit } => {
            to_json_binary(&query_auctions(deps, start_after, limit)?)?
        }
    };

    Ok(response)
}
