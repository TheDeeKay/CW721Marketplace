use crate::api::{AuctionId, AuctionResponse, AuctionsResponse, ConfigResponse, PriceAsset};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw721::Cw721ReceiveMsg;
use cw_utils::Duration;

#[cw_serde]
pub struct InstantiateMsg {
    /// NFT contract whose tokens are allowed in this auction contract.
    pub whitelisted_nft: String,
    /// Asset in which all the auctions created will be priced.
    pub price_asset: PriceAsset,
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
    ResolveEndedAuction {
        auction_id: AuctionId,
    },
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    CreateAuction {
        /// Duration of the auction. Once it ends, no new bids are accepted, and if there is
        /// an active bid, that bid wins the auction.
        duration: Duration,
        /// Minimum amount of funds to be accepted as the first bid
        minimum_bid_amount: Uint128,
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
    Auctions {},
}
