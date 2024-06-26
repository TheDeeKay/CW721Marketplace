use crate::api::{
    AuctionId, AuctionResponse, AuctionsResponse, ConfigResponse, PriceAssetUnchecked,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    /// NFT contract whose tokens are allowed in this auction contract.
    pub whitelisted_nft: String,
    /// Asset in which all the auctions created will be priced.
    pub price_asset: PriceAssetUnchecked,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Offer a bid on an auction for a single track.
    Bid {
        /// ID of the auction to bid on.
        auction_id: AuctionId,
        /// The amount of auction's price asset to bid.
        /// Required to be explicitly set, to avoid bugs from implicitly inferring bid amount
        /// from the funds received (especially if e.g. fees are added later on).
        bid_amount: Uint128,
    },

    /// Resolves an auction that has ended.
    /// This means that the auction's status will be changed, and assets will be resolved.
    ///
    /// If there is an active bid, the NFT will go to the bidder and the bid to the auction creator.
    /// If not, the NFT will go back to its owner.
    ResolveAuction {
        auction_id: AuctionId,
    },

    /// Cancels an active auction. Only callable by the auction creator.
    /// Will refund the active bid (if any), and send back the NFT to the creator.
    CancelAuction {
        auction_id: AuctionId,
    },

    Receive(Cw20ReceiveMsg),

    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum Cw20HookMsg {
    Bid {
        /// ID of the auction to bid on.
        auction_id: AuctionId,
        /// The amount of auction's price asset to bid.
        /// Required to be explicitly set, to avoid bugs from implicitly inferring bid amount
        /// from the funds received (especially if e.g. fees are added later on).
        bid_amount: Uint128,
    },
}

#[cw_serde]
pub enum Cw721HookMsg {
    CreateAuction {
        /// Duration of the auction. Once it ends, no new bids are accepted, and if there is
        /// an active bid, that bid wins the auction.
        duration: Duration,

        /// Minimum amount of funds to be accepted as the first bid
        minimum_bid_amount: Uint128,

        /// Optional amount to pay to instantly purchase the auction.
        ///
        /// If the amount is specified and someone bids that or higher amount, the auction
        /// ends immediately, and they win the bidding.
        buyout_price: Option<Uint128>,
    },
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AuctionResponse)]
    Auction { id: AuctionId },
    #[returns(AuctionsResponse)]
    Auctions {
        /// Whether the query should return active auctions.
        /// When set to false, will return finished auctions.
        active_auctions: bool,
        /// Optional parameter to start listing items after a certain ID (used for pagination)
        start_after: Option<AuctionId>,
        /// Optional parameter to limit the size of query response
        limit: Option<u32>,
    },
}
