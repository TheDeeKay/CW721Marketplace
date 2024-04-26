use crate::contract::{instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_json, Addr};
use tracks_auction_api::api::{Config, ConfigResponse};
use tracks_auction_api::msg::{InstantiateMsg, QueryMsg};

const ADMIN: &str = "admin";

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
        },
    )?;

    let response: ConfigResponse =
        from_json(query(deps.as_ref(), env.clone(), QueryMsg::Config {})?)?;

    assert_eq!(
        response.config,
        Config {
            whitelisted_nft: Addr::unchecked(whitelisted_nft)
        }
    );

    Ok(())
}
