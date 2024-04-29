use cosmwasm_std::StdError;
use cw_asset::AssetError;
use thiserror::Error;

pub type AuctionResult<T> = Result<T, AuctionError>;

#[derive(Error, Debug, PartialEq)]
pub enum AuctionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    AssetError(#[from] AssetError),

    #[error("CW721 you're using is not whitelisted for auctions")]
    Cw721NotWhitelisted,

    #[error("Duration has to be greater than 0")]
    InvalidAuctionDuration,

    // TODO: consolidate all the errors regarding invalid bid funds here, they're unnecessarily wide
    #[error("Attempting to bid with no funds")]
    NoBidFundsSupplied,

    #[error("No auction with given ID was found")]
    AuctionIdNotFound,

    #[error("Trying to send assets not necessary for the bid")]
    UnnecessaryAssetsForBid,

    #[error("Supplied funds do not match the attempted bid")]
    InsufficientFundsForBid,

    #[error("Bid is lower than minimum required")]
    BidLowerThanMinimum,

    #[error("Attempting to bid using the wrong asset")]
    BidWrongAsset,

    #[error("Cannot place a bid after the auction has ended")]
    BiddingAfterAuctionEnded,

    #[error("Auction is still in progress, cannot perform that action yet")]
    AuctionStillInProgress,
}
