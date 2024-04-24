use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response};
use cw721_tracks_api::api::TrackMetadata;
use cw721_tracks_api::error::TracksError;
use cw721_tracks_api::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

// Version info for migration
const CONTRACT_NAME: &str = "cw721-tracks";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

type Cw721MetadataContract<'a> = cw721_base::Cw721Contract<'a, TrackMetadata, Empty, Empty, Empty>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, TracksError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Cw721MetadataContract::default().instantiate(deps.branch(), env, info, msg)?)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, TracksError> {
    Cw721MetadataContract::default().execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, TracksError> {
    let response = Cw721MetadataContract::default().query(deps, env, msg)?;

    Ok(response)
}
