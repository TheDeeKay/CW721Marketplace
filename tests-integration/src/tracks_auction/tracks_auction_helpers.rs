use crate::helpers::{ImplApp, ADMIN};
use cosmwasm_std::{to_json_binary, Addr, BankMsg, BlockInfo, Coin, CosmosMsg, Uint128};
use cw721::Cw721ExecuteMsg::SendNft;
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor, IntoAddr};
use cw_utils::Duration;
use tracks_auction_api::api::{AuctionResponse, Bid, PriceAsset, PriceAssetUnchecked};
use tracks_auction_api::msg::QueryMsg::Auction;
use tracks_auction_api::msg::{ExecuteMsg as AuctionExecuteMsg, InstantiateMsg};
use AuctionExecuteMsg::ResolveAuction;
use BankMsg::Burn;
use CosmosMsg::Bank;

pub fn store_tracks_auction_code(app: &mut App) -> u64 {
    app.store_code(Box::new(ContractWrapper::new(
        tracks_auction::contract::execute,
        tracks_auction::contract::instantiate,
        tracks_auction::contract::query,
    )))
}

pub fn instantiate_tracks_auction(
    app: &mut App,
    code_id: u64,
    whitelisted_nft: String,
    price_asset: PriceAssetUnchecked,
) -> AnyResult<Addr> {
    let msg = InstantiateMsg {
        whitelisted_nft,
        price_asset,
    };

    app.instantiate_contract(
        code_id,
        ADMIN.into_addr(),
        &msg,
        &[],
        "Tracks auction",
        Some(ADMIN.to_string()),
    )
}

pub fn store_and_instantiate_tracks_auction(
    app: &mut App,
    whitelisted_nft: String,
    price_asset: PriceAssetUnchecked,
) -> AnyResult<(u64, Addr)> {
    let code_id = store_tracks_auction_code(app);
    let addr = instantiate_tracks_auction(app, code_id, whitelisted_nft, price_asset);

    addr.map(|address| (code_id, address))
}

pub fn create_nft_auction(
    app: &mut App,
    nft: Addr,
    auction: Addr,
    owner: &str,
    token_id: &str,
    duration: Duration,
    minimum_bid_amount: u128,
    buyout_price: Option<u128>,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        owner.into_addr(),
        nft,
        &SendNft {
            contract: auction.to_string(),
            token_id: token_id.to_string(),
            msg: to_json_binary(&tracks_auction_api::msg::Cw721HookMsg::CreateAuction {
                duration,
                minimum_bid_amount: Uint128::from(minimum_bid_amount),
                buyout_price: buyout_price.map(Uint128::from),
            })?,
        },
        &vec![],
    )
}

pub fn bid_on_auction(
    app: &mut App,
    auction: Addr,
    bidder: &str,
    auction_id: u64,
    bid: Coin,
) -> AnyResult<AppResponse> {
    app.mint_native(bidder, vec![bid.clone()])?;

    let result = app.execute_contract(
        bidder.into_addr(),
        auction,
        &AuctionExecuteMsg::Bid {
            auction_id,
            bid_amount: bid.amount,
        },
        &vec![bid.clone()],
    );

    // if bidding failed, burn what we minted to the bidder to make this 'atomic'
    if result.is_err() {
        app.execute(bidder.into_addr(), Bank(Burn { amount: vec![bid] }))?;
    }

    result
}

pub fn resolve_auction(
    app: &mut App,
    auction: Addr,
    sender: &str,
    auction_id: u64,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        sender.into_addr(),
        auction,
        &ResolveAuction { auction_id },
        &vec![],
    )
}

pub fn query_active_bid(app: &App, auction: Addr, auction_id: u64) -> AnyResult<Option<Bid>> {
    let auction: AuctionResponse = app
        .wrap()
        .query_wasm_smart(auction, &Auction { id: auction_id })?;

    Ok(auction.auction.active_bid)
}

pub fn assert_active_bid(
    app: &App,
    auction: Addr,
    auction_id: u64,
    bidder: &str,
    bid: Coin,
    posted_at: BlockInfo,
) {
    assert_eq!(
        query_active_bid(&app, auction, auction_id).unwrap(),
        Some(Bid {
            amount: bid.amount,
            asset: PriceAsset::native(bid.denom),
            bidder: bidder.into_addr(),
            posted_at,
        })
    );
}
