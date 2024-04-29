use crate::auctions::{
    load_auction, save_new_auction, update_active_bid, update_auction_status, CreateAuctionData,
};
use crate::config::load_config;
use cosmwasm_std::{
    from_json, wasm_execute, Addr, DepsMut, Env, MessageInfo, Response, StdError, SubMsg, Uint128,
};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ExecuteMsg::TransferNft;
use cw721::Cw721ReceiveMsg;
use cw_asset::Asset;
use cw_utils::Duration::{Height, Time};
use tracks_auction_api::api::AuctionStatus::{Active, Canceled, Resolved};
use tracks_auction_api::api::{AuctionId, Bid, PriceAsset, TrackAuction};
use tracks_auction_api::error::AuctionError::{
    AuctionCanceled, AuctionExpired, AuctionIdNotFound, AuctionResolved, AuctionStillInProgress,
    BidLowerThanMinimum, BidWrongAsset, BiddingAfterAuctionEnded, Cw721NotWhitelisted,
    InsufficientFundsForBid, InvalidAuctionDuration, NoBidFundsSupplied, Unauthorized,
    UnnecessaryAssetsForBid,
};
use tracks_auction_api::error::AuctionResult;
use tracks_auction_api::msg::Cw20HookMsg;
use tracks_auction_api::msg::Cw721HookMsg::CreateAuction;

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
            buyout_price,
        }) => {
            if duration == Time(0) || duration == Height(0) {
                return Err(InvalidAuctionDuration);
            }

            let creator = deps.api.addr_validate(&msg.sender)?;

            let auction_data = CreateAuctionData {
                duration,
                creator,
                nft_contract: info.sender,
                track_token_id: msg.token_id,
                minimum_bid_amount,
                buyout_price,
            };

            let id = save_new_auction(deps.storage, env.block, auction_data)?;

            Ok(Response::new()
                .add_attribute("action", "create_auction")
                .add_attribute("auction_id", id.to_string()))
        }
        _ => Err(StdError::generic_err("unknown NFT receive hook message").into()),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> AuctionResult<Response> {
    match from_json(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Bid {
            auction_id,
            bid_amount,
        }) => {
            let bid_asset = PriceAsset::cw20(info.sender.clone());
            let bid_received_amount = cw20_msg.amount;

            let bidder = deps.api.addr_validate(&cw20_msg.sender)?;
            resolve_bid(
                deps,
                env,
                bidder,
                auction_id,
                bid_asset,
                bid_received_amount,
                bid_amount,
            )
        }
        _ => Err(StdError::generic_err("unknown CW20 hook message received").into()),
    }
}

pub fn bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    auction_id: AuctionId,
    bid_amount: Uint128,
) -> AuctionResult<Response> {
    let bid_funds = match &info.funds[..] {
        [coin] => coin.clone(),
        [] => return Err(NoBidFundsSupplied),
        _ => return Err(UnnecessaryAssetsForBid),
    };

    let bid_asset = PriceAsset::native(&bid_funds.denom);
    let bid_received_amount = bid_funds.amount;

    resolve_bid(
        deps,
        env,
        info.sender,
        auction_id,
        bid_asset,
        bid_received_amount,
        bid_amount,
    )
}

fn resolve_bid(
    deps: DepsMut,
    env: Env,
    bidder: Addr,
    auction_id: AuctionId,
    bid_asset: PriceAsset,
    bid_received_amount: Uint128,
    bid_amount: Uint128,
) -> AuctionResult<Response> {
    // TODO: sending more funds than the bid states will pass this check. it's debatable whether that should happen
    if bid_received_amount < bid_amount {
        return Err(InsufficientFundsForBid);
    }

    let auction = load_auction(deps.storage, auction_id)?.ok_or(AuctionIdNotFound)?;

    // do not allow self-bidding
    if bidder == auction.creator {
        return Err(Unauthorized);
    }

    if auction.has_ended(&env.block) {
        return Err(BiddingAfterAuctionEnded);
    }

    let config = load_config(deps.storage)?;

    if config.price_asset != bid_asset {
        return Err(BidWrongAsset);
    } else if bid_amount < auction.minimum_next_bid_amount() {
        return Err(BidLowerThanMinimum);
    }

    // if buyout price is reached, end the auction here
    if let Some(buyout_price) = auction.buyout_price {
        if buyout_price <= bid_amount {
            return buyout_auction(deps, auction, bidder, bid_asset, bid_amount);
        }
    }

    update_active_bid(
        deps.storage,
        auction_id,
        Bid {
            amount: bid_amount,
            asset: config.price_asset,
            bidder,
            posted_at: env.block,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "bid")
        .add_attribute("auction_id", auction_id.to_string())
        .add_attribute("bid_amount", bid_amount.to_string())
        // if there was an active bid before this, refund it
        .add_submessages(refund_previous_bid_msg(&auction)?))
}

fn buyout_auction(
    deps: DepsMut,
    auction: TrackAuction,
    bidder: Addr,
    bid_asset: PriceAsset,
    bid_amount: Uint128,
) -> AuctionResult<Response> {
    let send_nft_to_buyer_msg =
        transfer_nft_msg(&auction.nft_contract, bidder, &auction.track_token_id)?;

    let refund_previous_bid_msg = refund_previous_bid_msg(&auction)?;

    let send_bid_amount_msg = SubMsg::new(
        Asset::new(bid_asset.to_asset_info(), bid_amount).transfer_msg(auction.creator)?,
    );

    update_auction_status(deps.storage, auction.id, Resolved)?;

    Ok(Response::new()
        .add_attribute("action", "instant_buyout")
        .add_attribute("auction_id", auction.id.to_string())
        .add_attribute("bid_amount", bid_amount.to_string())
        .add_submessage(send_bid_amount_msg)
        .add_submessage(send_nft_to_buyer_msg)
        .add_submessages(refund_previous_bid_msg))
}

pub fn resolve_auction(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    auction_id: AuctionId,
) -> AuctionResult<Response> {
    let auction = load_auction(deps.storage, auction_id)?.ok_or(AuctionIdNotFound)?;

    if !auction.has_ended(&env.block) {
        return Err(AuctionStillInProgress);
    }

    if auction.status == Resolved {
        return Err(AuctionResolved);
    }

    match auction.status {
        Resolved => {
            return Err(AuctionResolved);
        }
        Canceled => {
            return Err(AuctionCanceled);
        }
        Active => {
            // no-op
        }
    }

    update_auction_status(deps.storage, auction_id, Resolved)?;

    let base_response = Response::new()
        .add_attribute("action", "resolve_auction")
        .add_attribute("auction_id", auction_id.to_string());

    match auction.active_bid {
        Some(bid) => {
            // send NFT to the highest bidder
            let send_nft_to_bidder_msg =
                transfer_nft_msg(auction.nft_contract, bid.bidder, auction.track_token_id)?;
            // send funds to the auction creator
            let send_bid_to_auction_creator_msg = SubMsg::new(
                Asset::new(bid.asset.to_asset_info(), bid.amount).transfer_msg(auction.creator)?,
            );

            Ok(base_response
                .add_submessage(send_nft_to_bidder_msg)
                .add_submessage(send_bid_to_auction_creator_msg))
        }
        None => {
            // received no bids, simply return the NFT to the auction creator
            let return_nft_submsg = transfer_nft_msg(
                auction.nft_contract,
                auction.creator,
                auction.track_token_id,
            )?;
            Ok(base_response.add_submessage(return_nft_submsg))
        }
    }
}

pub fn cancel_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    auction_id: AuctionId,
) -> AuctionResult<Response> {
    let auction = load_auction(deps.storage, auction_id)?.ok_or(AuctionIdNotFound)?;

    if auction.creator != info.sender {
        return Err(Unauthorized);
    }

    match auction.status {
        Resolved => return Err(AuctionResolved),
        Canceled => return Err(AuctionCanceled),
        Active => {
            // no-op
        }
    }

    if auction.has_ended(&env.block) {
        return Err(AuctionExpired);
    }

    let refund_previous_bid_msg = refund_previous_bid_msg(&auction)?;

    update_auction_status(deps.storage, auction_id, Canceled)?;

    let send_nft_back_submsg = transfer_nft_msg(
        auction.nft_contract,
        auction.creator,
        auction.track_token_id,
    )?;
    Ok(Response::new()
        .add_attribute("action", "cancel_auction")
        .add_attribute("auction_id", auction_id.to_string())
        .add_submessage(send_nft_back_submsg)
        .add_submessages(refund_previous_bid_msg))
}

fn refund_previous_bid_msg(auction: &TrackAuction) -> AuctionResult<Vec<SubMsg>> {
    let refund_previous_bid_msg = match &auction.active_bid {
        Some(bid) => vec![SubMsg::new(
            Asset::new(bid.asset.to_asset_info(), bid.amount)
                .transfer_msg(bid.bidder.to_string())?,
        )],
        None => vec![],
    };
    Ok(refund_previous_bid_msg)
}

fn transfer_nft_msg(
    nft_contract: impl Into<String>,
    recipient: impl Into<String>,
    token_id: impl Into<String>,
) -> AuctionResult<SubMsg> {
    Ok(SubMsg::new(wasm_execute(
        nft_contract.into(),
        &TransferNft {
            recipient: recipient.into(),
            token_id: token_id.into(),
        },
        vec![],
    )?))
}
