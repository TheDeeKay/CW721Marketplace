use crate::api::AuctionsResponse;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;
use cw721::Cw721ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    /// List of NFT contracts whose tokens are allowed in this auction contract.
    pub whitelisted_nfts: Vec<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
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
    #[returns(AuctionsResponse)]
    Auctions {},
}
