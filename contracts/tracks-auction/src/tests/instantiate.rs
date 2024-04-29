use crate::query::query_config;
use crate::tests::helpers::{instantiate_with_native_price_asset, ADMIN, UATOM};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{attr, Addr};
use tracks_auction_api::api::{Config, PriceAsset};

#[test]
fn instantiate_stores_config() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let whitelisted_nft = "nft_contract";

    let response = instantiate_with_native_price_asset(
        deps.as_mut(),
        env.clone(),
        ADMIN,
        whitelisted_nft,
        UATOM,
    )?;

    assert_eq!(
        response.attributes,
        vec![
            attr("action", "instantiate"),
            attr("whitelisted_nft", whitelisted_nft),
            attr("price_asset", "native"),
            attr("price_asset_denom", UATOM),
        ],
    );

    assert_eq!(
        query_config(deps.as_ref())?.config,
        Config {
            whitelisted_nft: Addr::unchecked(whitelisted_nft),
            price_asset: PriceAsset::native(UATOM),
        }
    );

    Ok(())
}
