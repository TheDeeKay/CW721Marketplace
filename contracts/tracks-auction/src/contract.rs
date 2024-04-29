use crate::auctions::{load_auction, save_new_auction, update_active_bid};
use crate::config::{load_config, save_config};
use crate::query::{query_auction, query_auctions, query_config};
use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, SubMsg, Uint128,
};
use cw721::Cw721ReceiveMsg;
use cw_asset::Asset;
use cw_utils::Duration::{Height, Time};
use tracks_auction_api::api::{AuctionId, Bid, Config, PriceAsset};
use tracks_auction_api::error::AuctionError::{
    AuctionIdNotFound, BidLowerThanMinimum, BidWrongAsset, BiddingAfterAuctionEnded,
    Cw721NotWhitelisted, InsufficientFundsForBid, InvalidAuctionDuration, NoBidFundsSupplied,
    UnnecessaryAssetsForBid,
};
use tracks_auction_api::error::{AuctionError, AuctionResult};
use tracks_auction_api::msg::{Cw721HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use Cw721HookMsg::CreateAuction;
use ExecuteMsg::ReceiveNft;
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
        ReceiveNft(nft_msg) => receive_nft(deps, env, info, nft_msg),
        ExecuteMsg::Bid {
            auction_id,
            bid_amount,
        } => bid(deps, env, info, auction_id, bid_amount),
    }
}

// TODO: move to another file
pub fn receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw721ReceiveMsg,
) -> AuctionResult<Response> {
    let config = load_config(deps.storage)?;

    // only whitelisted NFT can call this
    if info.sender != config.whitelisted_nft {
        return Err(Cw721NotWhitelisted);
    }

    match from_json(msg.msg) {
        Ok(CreateAuction {
            duration,
            minimum_bid_amount,
        }) => {
            if duration == Time(0) || duration == Height(0) {
                return Err(InvalidAuctionDuration);
            }
            let submitter = deps.api.addr_validate(&msg.sender)?;
            save_new_auction(
                deps.storage,
                env.block,
                duration,
                submitter,
                msg.token_id,
                minimum_bid_amount,
            )?;
            Ok(Response::new()) // TODO: add attributes
        }
        _ => Err(StdError::generic_err("unknown NFT receive hook message").into()),
    }
}

// TODO: move to another file
pub fn bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    auction_id: AuctionId,
    bid_amount: Uint128,
) -> AuctionResult<Response> {
    // TODO: refactor

    let auction = load_auction(deps.storage, auction_id)?;

    match auction {
        None => Err(AuctionIdNotFound),
        Some(auction) => {
            if auction.has_ended(env.block.clone()) {
                return Err(BiddingAfterAuctionEnded);
            }

            match &info.funds[..] {
                [coin] => {
                    // TODO: should we really accept more funds than specified? should be a design decision
                    if coin.amount < bid_amount {
                        Err(InsufficientFundsForBid)
                    } else {
                        let config = load_config(deps.storage)?;
                        if config.price_asset != PriceAsset::native(&coin.denom) {
                            Err(BidWrongAsset)
                        } else if auction.minimum_next_bid_amount() <= bid_amount {
                            let last_active_bid = update_active_bid(
                                deps.storage,
                                auction_id,
                                Bid {
                                    amount: bid_amount,
                                    asset: config.price_asset,
                                    bidder: info.sender,
                                    posted_at: env.block,
                                },
                            )?;

                            let base_response = Response::new(); // TODO: add attributes

                            match last_active_bid {
                                Some(bid) => {
                                    let refund_bid_submsg = SubMsg::new(
                                        Asset::native(&coin.denom, bid.amount)
                                            .transfer_msg(bid.bidder)?,
                                    );
                                    Ok(base_response.add_submessage(refund_bid_submsg))
                                }
                                None => Ok(base_response),
                            }
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
        Auction { id } => to_json_binary(&query_auction(deps, id)?)?,
        Auctions {} => to_json_binary(&query_auctions(deps)?)?,
    };

    Ok(response)
}
