use crate::query::query_config;
use crate::tests::helpers::{instantiate_with_native_price_asset, ADMIN, UATOM};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::Addr;
use tracks_auction_api::api::{Config, PriceAsset};

#[test]
fn instantiate_stores_config() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let whitelisted_nft = "nft_contract";

    instantiate_with_native_price_asset(deps.as_mut(), env.clone(), ADMIN, whitelisted_nft, UATOM)?;

    let response = query_config(deps.as_ref())?;

    assert_eq!(
        response.config,
        Config {
            whitelisted_nft: Addr::unchecked(whitelisted_nft),
            price_asset: PriceAsset::native(UATOM),
        }
    );

    Ok(())
}
