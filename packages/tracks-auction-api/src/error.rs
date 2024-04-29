use cosmwasm_std::StdError;
use thiserror::Error;

pub type AuctionResult<T> = Result<T, AuctionError>;

#[derive(Error, Debug, PartialEq)]
pub enum AuctionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("CW721 you're using is not whitelisted for auctions")]
    Cw721NotWhitelisted,

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
}
