use crate::contract::instantiate;
use crate::query::query_config;
use crate::tests::helpers::{ADMIN, UATOM};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::Addr;
use tracks_auction_api::api::{Config, PriceAsset};
use tracks_auction_api::msg::InstantiateMsg;

#[test]
fn instantiate_stores_config() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let whitelisted_nft = "nft_contract";

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        InstantiateMsg {
            whitelisted_nft: whitelisted_nft.to_string(),
            price_asset: PriceAsset::native(UATOM),
        },
    )?;

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
