use crate::api::{AuctionId, AuctionsResponse, ConfigResponse, PriceAsset};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw721::Cw721ReceiveMsg;

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
    },
    ReceiveNft(Cw721ReceiveMsg),
}

#[cw_serde]
pub enum Cw721HookMsg {
    CreateAuction {
        /// Minimum amount of funds to be accepted as the first bid
        minimum_bid_amount: Uint128,
    },
}

#[derive(QueryResponses)]
#[cw_serde]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(AuctionsResponse)]
    Auctions {},
}
