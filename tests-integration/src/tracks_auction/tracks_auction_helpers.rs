use crate::helpers::{ImplApp, TestFixture, ADMIN};
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

pub trait TracksAuctionExecute {
    fn create_nft_auction(
        &mut self,
        owner: &str,
        token_id: &str,
        duration: Duration,
        minimum_bid_amount: u128,
        buyout_price: Option<u128>,
    ) -> AnyResult<AppResponse>;

    fn bid_on_auction(
        &mut self,
        bidder: &str,
        auction_id: u64,
        bid: Coin,
    ) -> AnyResult<AppResponse>;

    fn cancel_auction(
        &mut self,
        sender: &str,
        auction_id: u64,
    ) -> AnyResult<AppResponse>;

    fn resolve_auction(&mut self, sender: &str, auction_id: u64) -> AnyResult<AppResponse>;
}

impl TracksAuctionExecute for TestFixture {
    fn create_nft_auction(
        &mut self,
        owner: &str,
        token_id: &str,
        duration: Duration,
        minimum_bid_amount: u128,
        buyout_price: Option<u128>,
    ) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            owner.into_addr(),
            self.cw721_tracks.addr.clone(),
            &SendNft {
                contract: self.tracks_auction.addr.to_string(),
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

    fn bid_on_auction(
        &mut self,
        bidder: &str,
        auction_id: u64,
        bid: Coin,
    ) -> AnyResult<AppResponse> {
        self.app.mint_native(bidder, vec![bid.clone()])?;

        let result = self.app.execute_contract(
            bidder.into_addr(),
            self.tracks_auction.addr.clone(),
            &AuctionExecuteMsg::Bid {
                auction_id,
                bid_amount: bid.amount,
            },
            &vec![bid.clone()],
        );

        // if bidding failed, burn what we minted to the bidder to make this 'atomic'
        if result.is_err() {
            self.app
                .execute(bidder.into_addr(), Bank(Burn { amount: vec![bid] }))?;
        }

        result
    }

    fn cancel_auction(&mut self, sender: &str, auction_id: u64) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            sender.into_addr(),
            self.tracks_auction.addr.clone(),
            &AuctionExecuteMsg::CancelAuction {
                auction_id,
            },
            &vec![],
        )
    }

    fn resolve_auction(&mut self, sender: &str, auction_id: u64) -> AnyResult<AppResponse> {
        self.app.execute_contract(
            sender.into_addr(),
            self.tracks_auction.addr.clone(),
            &ResolveAuction { auction_id },
            &vec![],
        )
    }
}

pub trait TracksAuctionQuery {
    fn query_active_bid(&self, auction_id: u64) -> AnyResult<Option<Bid>>;
    fn assert_active_bid(
        &self,
        auction_id: u64,
        bidder: &str,
        bid: Coin,
        posted_at: Option<BlockInfo>,
    );
}

impl TracksAuctionQuery for TestFixture {
    fn query_active_bid(&self, auction_id: u64) -> AnyResult<Option<Bid>> {
        let auction: AuctionResponse = self.app.wrap().query_wasm_smart(
            self.tracks_auction.addr.to_string(),
            &Auction { id: auction_id },
        )?;

        Ok(auction.auction.active_bid)
    }

    fn assert_active_bid(
        &self,
        auction_id: u64,
        bidder: &str,
        bid: Coin,
        posted_at: Option<BlockInfo>,
    ) {
        assert_eq!(
            self.query_active_bid(auction_id).unwrap(),
            Some(Bid {
                amount: bid.amount,
                asset: PriceAsset::native(bid.denom),
                bidder: bidder.into_addr(),
                posted_at: posted_at.unwrap_or(self.app.block_info()),
            })
        );
    }
}
