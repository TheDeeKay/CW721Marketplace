use cosmwasm_std::StdError;
use thiserror::Error;

pub type AuctionResult<T> = Result<T, AuctionError>;

#[derive(Error, Debug, PartialEq)]
pub enum AuctionError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("CW721 you're using is not whitelisted for auctions")]
    Cw721NotWhitelisted,
}
