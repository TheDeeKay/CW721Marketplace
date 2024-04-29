use crate::config::save_config;
use crate::execute;
use crate::execute::{bid, receive_cw20};
use crate::query::{query_auction, query_auctions, query_config};
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
};
use execute::{cancel_auction, receive_nft, resolve_auction};
use tracks_auction_api::api::{Config, PriceAsset};
use tracks_auction_api::error::{AuctionError, AuctionResult};
use tracks_auction_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use ExecuteMsg::{Bid, CancelAuction, Receive, ReceiveNft, ResolveAuction};
use PriceAsset::{Cw20, Native};
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
        whitelisted_nft: nft_addr.clone(),
        price_asset: msg.price_asset.check(deps.api)?,
    };
    save_config(deps.storage, &config)?;

    let price_asset_attributes = match &config.price_asset {
        Native { denom } => vec![("price_asset", "native"), ("price_asset_denom", denom)],
        Cw20 { contract } => vec![
            ("price_asset", "cw20"),
            ("price_asset_cw20_contract", contract.as_ref()),
        ],
    };

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("whitelisted_nft", nft_addr.to_string())
        .add_attributes(price_asset_attributes))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> AuctionResult<Response> {
    match msg {
        Receive(cw20_msg) => receive_cw20(deps, env, info, cw20_msg),
        ReceiveNft(nft_msg) => receive_nft(deps, env, info, nft_msg),
        Bid {
            auction_id,
            bid_amount,
        } => bid(deps, env, info, auction_id, bid_amount),
        ResolveAuction { auction_id } => resolve_auction(deps, env, info, auction_id),
        CancelAuction { auction_id } => cancel_auction(deps, env, info, auction_id),
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
