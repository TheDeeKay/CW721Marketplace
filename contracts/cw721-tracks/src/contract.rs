use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
use cw721_base::state::TokenInfo;
use cw721_base::ContractError;
use cw721_tracks_api::api::TrackMetadata;
use cw721_tracks_api::error::TracksError;
use cw721_tracks_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// Version info for migration
const CONTRACT_NAME: &str = "cw721-tracks";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Cw721TrackMetadataContract<'a> =
    cw721_base::Cw721Contract<'a, TrackMetadata, Empty, Empty, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, TracksError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // set the minter to this contract, as this doesn't accept None
    // this parameter should be meaningless in our case, as minting is permissionless
    let minter = env.contract.address.to_string();

    Ok(Cw721TrackMetadataContract::default().instantiate(
        deps.branch(),
        env,
        info,
        cw721_base::InstantiateMsg {
            name: msg.name,
            symbol: msg.symbol,
            minter,
        },
    )?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, TracksError> {
    let contract = Cw721TrackMetadataContract::default();
    match msg {
        ExecuteMsg::Mint {
            token_id,
            owner,
            token_uri,
            extension,
        } => {
            // create the token
            let token = TokenInfo {
                owner: deps.api.addr_validate(&owner)?,
                approvals: vec![],
                token_uri,
                extension,
            };
            contract
                .tokens
                .update(deps.storage, &token_id, |old| match old {
                    Some(_) => Err(ContractError::Claimed {}),
                    None => Ok(token),
                })?;

            contract.increment_tokens(deps.storage)?;

            Ok(Response::new()
                .add_attribute("action", "mint")
                .add_attribute("minter", info.sender)
                .add_attribute("owner", owner)
                .add_attribute("token_id", token_id))
        }
        _ => contract.execute(deps, env, info, msg),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, TracksError> {
    let response = Cw721TrackMetadataContract::default().query(deps, env, msg)?;

    Ok(response)
}
