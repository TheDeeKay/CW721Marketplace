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

    #[error("Unauthorized")]
    Unauthorized,

    #[error("CW721 you're using is not whitelisted for auctions")]
    Cw721NotWhitelisted,

    #[error("Duration has to be greater than 0")]
    InvalidAuctionDuration,

    #[error("No auction with given ID was found")]
    AuctionIdNotFound,

    #[error("Attempting to bid with no funds")]
    NoBidFundsSupplied,

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

    #[error("Auction was already resolved")]
    AuctionResolved,

    #[error("Auction was already canceled")]
    AuctionCanceled,

    #[error("Auction has already expired")]
    AuctionExpired,
}
