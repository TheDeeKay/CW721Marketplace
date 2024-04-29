use crate::auctions::{load_auction, save_auction};
use crate::config::{load_config, save_config};
use crate::query::{query_auctions, query_config};
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, Uint128,
};
use cw721::Cw721ReceiveMsg;
use tracks_auction_api::api::{AuctionId, Config, PriceAsset};
use tracks_auction_api::error::AuctionError::{
    AuctionIdNotFound, BidLowerThanMinimum, BidWrongAsset, Cw721NotWhitelisted,
    InsufficientFundsForBid, NoBidFundsSupplied, UnnecessaryAssetsForBid,
};
use tracks_auction_api::error::{AuctionError, AuctionResult};
use tracks_auction_api::msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use Cw721HookMsg::CreateAuction;
use ExecuteMsg::{Bid, ReceiveNft};
use QueryMsg::Auctions;

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
        ReceiveNft(nft_msg) => receive_nft(deps, env, info, nft_msg),
        Bid {
            auction_id,
            bid_amount,
        } => bid(deps, env, info, auction_id, bid_amount),
    }
}

// TODO: move to another file
pub fn receive_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> AuctionResult<Response> {
    let config = load_config(deps.storage)?;

    // only whitelisted NFT can call this
    if info.sender != config.whitelisted_nft {
        return Err(Cw721NotWhitelisted);
    }

    match from_json(msg.msg) {
        Ok(CreateAuction { minimum_bid_amount }) => {
            let submitter = deps.api.addr_validate(&msg.sender)?;
            save_auction(deps.storage, submitter, msg.token_id, minimum_bid_amount)?;
            Ok(Response::new()) // TODO: add attributes
        }
        _ => Err(StdError::generic_err("unknown NFT receive hook message").into()),
    }
}

// TODO: move to another file
pub fn bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    auction_id: AuctionId,
    bid_amount: Uint128, // TODO: use (provoke by tests)
) -> AuctionResult<Response> {
    // TODO: refactor

    let auction = load_auction(deps.storage, auction_id)?;

    match auction {
        None => Err(AuctionIdNotFound),
        Some(_) => {
            match &info.funds[..] {
                [coin] => {
                    // TODO: should we really accept more funds than specified?
                    if coin.amount < bid_amount {
                        Err(InsufficientFundsForBid)
                    } else {
                        let config = load_config(deps.storage)?;
                        if config.price_asset != PriceAsset::native(&coin.denom) {
                            Err(BidWrongAsset)
                        } else {
                            Err(BidLowerThanMinimum)
                        }
                    }
                }
                [] => Err(NoBidFundsSupplied),
                _ => Err(UnnecessaryAssetsForBid),
            }
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, AuctionError> {
    let response = match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?)?,
        Auctions {} => to_json_binary(&query_auctions(deps)?)?,
    };

    Ok(response)
}
