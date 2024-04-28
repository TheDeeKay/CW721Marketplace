use crate::query::query_auctions;
use crate::tests::helpers::{
    create_test_auction, instantiate_with_native_price_asset, ADMIN, NFT_ADDR, NFT_ADDR2, TOKEN1,
    UANDR, UATOM, USER1,
};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::Addr;
use tracks_auction_api::api::{PriceAsset, TrackAuction};
use tracks_auction_api::error::AuctionError::Cw721NotWhitelisted;

#[test]
fn create_auction_for_non_whitelisted_nft_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UANDR)?;

    let result = create_test_auction(deps.as_mut(), env.clone(), NFT_ADDR2, TOKEN1, ADMIN, 0);

    assert_eq!(result, Err(Cw721NotWhitelisted));

    Ok(())
}

#[test]
fn create_auction_saves_it_with_relevant_data() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, NFT_ADDR, UATOM)?;

    let track_token_id = "first_track";

    create_test_auction(
        deps.as_mut(),
        env.clone(),
        NFT_ADDR,
        track_token_id,
        USER1,
        4,
    )?;

    let response = query_auctions(deps.as_ref())?;

    assert_eq!(
        response.auctions,
        vec![TrackAuction {
            id: 0,
            submitter: Addr::unchecked(USER1),
            track_token_id: track_token_id.to_string(),
            minimum_bid_amount: 4u8.into(),
            price_asset: PriceAsset::native("uatom"),
        }]
    );

    Ok(())
}
