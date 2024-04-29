use crate::tests::helpers::{
    after_seconds, create_test_auction, default_duration, instantiate_with_native_price_asset,
    test_bid, test_cancel_auction, test_resolve_auction, ADMIN, NFT_ADDR, TOKEN1, UANDR, USER1,
    USER2,
};
use cosmwasm_std::coins;
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cw_utils::Duration::Time;
use tracks_auction_api::error::AuctionError::{
    AuctionAlreadyResolved, AuctionExpired, Unauthorized,
};

#[test]
fn cancel_auction_only_permitted_to_auction_creator() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        default_duration(),
        5,
    )?;

    let result = test_cancel_auction(deps.as_mut(), env, USER2, 0);

    assert_eq!(result, Err(Unauthorized));

    Ok(())
}

#[test]
fn cancel_auction_on_expired_auction_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Time(20),
        5,
    )?;

    let result = test_cancel_auction(deps.as_mut(), after_seconds(&env, 21), USER1, 0);

    assert_eq!(result, Err(AuctionExpired));

    Ok(())
}

#[test]
fn cancel_auction_on_resolved_auction_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        TOKEN1,
        USER1,
        Time(20),
        5,
    )?;

    test_bid(deps.as_mut(), env.clone(), USER2, 0, 5, &coins(5, UANDR))?;

    let env_after_expiration = after_seconds(&env, 21);

    test_resolve_auction(deps.as_mut(), env_after_expiration.clone(), ADMIN, 0)?;

    let result = test_cancel_auction(deps.as_mut(), env_after_expiration, USER1, 0);

    assert_eq!(result, Err(AuctionAlreadyResolved));

    Ok(())
}
