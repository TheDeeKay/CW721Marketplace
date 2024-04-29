use crate::helpers::ADMIN;
use cosmwasm_std::Addr;
use cw721::Cw721QueryMsg::NftInfo;
use cw721::{NftInfoResponse, OwnerOfResponse, TokensResponse};
use cw721_tracks_api::api::TrackMetadata;
use cw721_tracks_api::msg::ExecuteMsg;
use cw_multi_test::error::AnyResult;
use cw_multi_test::{App, AppResponse, ContractWrapper, Executor, IntoAddr};

pub fn store_cw721_tracks_code(app: &mut App) -> u64 {
    app.store_code(Box::new(ContractWrapper::new(
        cw721_tracks::contract::execute,
        cw721_tracks::contract::instantiate,
        cw721_tracks::contract::query,
    )))
}

pub fn instantiate_cw721_tracks(app: &mut App, code_id: u64) -> AnyResult<Addr> {
    let msg = cw721_tracks_api::msg::InstantiateMsg {
        name: "CW721 tracks".to_string(),
        symbol: "TRKS".to_string(),
    };

    app.instantiate_contract(
        code_id,
        ADMIN.into_addr(),
        &msg,
        &[],
        "Tracks auction",
        Some(ADMIN.into_addr().to_string()),
    )
}

pub fn store_and_instantiate_cw721_tracks(app: &mut App) -> AnyResult<(u64, Addr)> {
    let code_id = store_cw721_tracks_code(app);
    let addr = instantiate_cw721_tracks(app, code_id);

    addr.map(|address| (code_id, address))
}

pub fn mint_nft(
    app: &mut App,
    nft: Addr,
    owner: &str,
    token_id: &str,
    token_uri: Option<&str>,
    metadata: TrackMetadata,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        owner.into_addr(),
        nft,
        &ExecuteMsg::Mint {
            token_id: token_id.to_string(),
            owner: owner.into_addr().to_string(),
            token_uri: token_uri.map(|it| it.to_string()),
            extension: metadata,
        },
        &vec![],
    )
}

pub trait NftQueries {
    fn query_nft(&self, nft: Addr, token_id: &str) -> AnyResult<NftInfoResponse<TrackMetadata>>;
    fn query_nfts(&self, nft: Addr, start_after: Option<String>) -> AnyResult<Vec<String>>;

    fn query_nft_owner(&self, nft: Addr, token_id: &str) -> AnyResult<String>;
    fn assert_nft_owner(&self, nft: Addr, token_id: &str, expected_owner: &str);
}

impl NftQueries for App {
    fn query_nft(&self, nft: Addr, token_id: &str) -> AnyResult<NftInfoResponse<TrackMetadata>> {
        let response: NftInfoResponse<TrackMetadata> = self.wrap().query_wasm_smart(
            nft,
            &NftInfo {
                token_id: token_id.to_string(),
            },
        )?;
        Ok(response)
    }

    fn query_nfts(&self, nft: Addr, start_after: Option<String>) -> AnyResult<Vec<String>> {
        let response: TokensResponse = self.wrap().query_wasm_smart(
            nft,
            &cw721::Cw721QueryMsg::AllTokens {
                start_after,
                limit: None,
            },
        )?;
        Ok(response.tokens)
    }

    fn query_nft_owner(&self, nft: Addr, token_id: &str) -> AnyResult<String> {
        let response: OwnerOfResponse = self.wrap().query_wasm_smart(
            nft,
            &cw721::Cw721QueryMsg::OwnerOf {
                token_id: token_id.into(),
                include_expired: None,
            },
        )?;
        Ok(response.owner)
    }

    fn assert_nft_owner(&self, nft: Addr, token_id: &str, expected_owner: &str) {
        let nft_owner = self.query_nft_owner(nft, token_id).unwrap();
        assert_eq!(nft_owner, expected_owner.into_addr().to_string())
    }
}

pub fn query_nft_owner(app: &App, nft: Addr, token_id: impl Into<String>) -> AnyResult<String> {
    let response: OwnerOfResponse = app.wrap().query_wasm_smart(
        nft,
        &cw721::Cw721QueryMsg::OwnerOf {
            token_id: token_id.into(),
            include_expired: None,
        },
    )?;
    Ok(response.owner)
}
