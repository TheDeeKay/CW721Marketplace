use crate::contract::{execute, instantiate, Cw721TrackMetadataContract};
use cosmwasm_std::attr;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cw721::Cw721Query;
use cw721_base::{ContractError, ExecuteMsg};
use cw721_tracks_api::api::{AlbumMetadata, TrackMetadata};
use cw721_tracks_api::msg::InstantiateMsg;
use ContractError::Claimed;
use ExecuteMsg::Mint;

const ADMIN: &str = "admin";

const USER1: &str = "user1";
const USER2: &str = "user2";

#[test]
fn mint_creates_new_token() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        anonymous_instantiate_msg(),
    )?;

    let token_id = "Boden's first track ID";
    let token_uri = Some("www.sleeper.com".to_string());
    let track_metadata = anonymous_track_metadata();

    let response = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(USER1, &vec![]),
        Mint {
            token_id: token_id.to_string(),
            owner: USER2.to_string(),
            token_uri: token_uri.clone(),
            extension: track_metadata.clone(),
        },
    )?;

    assert_eq!(
        response.attributes,
        vec![
            attr("action", "mint"),
            attr("minter", USER1),
            attr("owner", USER2),
            attr("token_id", token_id),
        ]
    );

    let contract = Cw721TrackMetadataContract::default();

    let nft_info = contract.nft_info(deps.as_ref(), token_id.to_string())?;

    assert_eq!(nft_info.token_uri, token_uri);
    assert_eq!(nft_info.extension, track_metadata);

    let num_tokens = contract.num_tokens(deps.as_ref())?;
    assert_eq!(num_tokens.count, 1u64);

    let owner = contract.owner_of(deps.as_ref(), env.clone(), token_id.to_string(), false)?;
    assert_eq!(owner.owner, USER2.to_string());
    assert!(owner.approvals.is_empty());

    Ok(())
}

#[test]
fn mint_existing_id_fails() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        anonymous_instantiate_msg(),
    )?;

    let token_id = "Random ID";

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(USER1, &vec![]),
        Mint {
            token_id: token_id.to_string(),
            owner: USER1.to_string(),
            token_uri: None,
            extension: anonymous_track_metadata(),
        },
    )?;

    let mint_existing_id_result = execute(
        deps.as_mut(),
        env.clone(),
        mock_info(USER2, &vec![]),
        Mint {
            token_id: token_id.to_string(),
            owner: USER2.to_string(),
            token_uri: None,
            extension: TrackMetadata {
                artist_name: "Different artist".to_string(),
                track_name: "Different track name".to_string(),
                ..anonymous_track_metadata()
            },
        },
    );

    assert_eq!(mint_existing_id_result, Err(Claimed {}));

    Ok(())
}

#[test]
fn minting_is_permissionless() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    let env = mock_env();

    instantiate(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN, &vec![]),
        anonymous_instantiate_msg(),
    )?;

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("random minter", &vec![]),
        Mint {
            token_id: "1".to_string(),
            owner: "random owner".to_string(),
            token_uri: None,
            extension: anonymous_track_metadata(),
        },
    )?;

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("another random minter", &vec![]),
        Mint {
            token_id: "2".to_string(),
            owner: "another random owner".to_string(),
            token_uri: None,
            extension: anonymous_track_metadata(),
        },
    )?;

    let contract = Cw721TrackMetadataContract::default();

    assert_eq!(contract.num_tokens(deps.as_ref())?.count, 2u64);

    Ok(())
}

fn anonymous_instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        name: "Track NFTs".to_string(),
        symbol: "TRKS".to_string(),
    }
}

fn anonymous_track_metadata() -> TrackMetadata {
    TrackMetadata {
        artist_name: "Boden".to_string(),
        album: Some(AlbumMetadata {
            name: "Presidency".to_string(),
            artwork_url: None,
            year: Some(2020u64.into()),
        }),
        track_name: "Debt Spiral".to_string(),
        audio_track_url: "https://www.usdebtclock.org/".to_string(),
    }
}
