use crate::contract::{instantiate, query};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_json, Addr};
use tracks_auction_api::api::NftWhitelistResponse;
use tracks_auction_api::msg::{InstantiateMsg, QueryMsg};
use QueryMsg::NftWhitelist;

const ADMIN: &str = "admin";

#[test]
fn instantiate_stores_whitelisted_nfts() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    let nft1 = "nft_contract1";
    let nft2 = "nft_contract2";

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        InstantiateMsg {
            whitelisted_nfts: vec![nft1.to_string(), nft2.to_string()],
        },
    )?;

    let response: NftWhitelistResponse =
        from_json(query(deps.as_ref(), env.clone(), NftWhitelist {})?)?;

    assert_eq!(
        response.nft_whitelist,
        vec![Addr::unchecked(nft1), Addr::unchecked(nft2)]
    );

    Ok(())
}
