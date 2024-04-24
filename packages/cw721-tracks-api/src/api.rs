use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint64;

#[cw_serde]
pub struct TrackMetadata {
    pub artist_name: String,
    pub album: Option<AlbumMetadata>,
    pub track_name: String,
    pub audio_track_url: String,
}

#[cw_serde]
pub struct AlbumMetadata {
    pub name: String,
    pub artwork_url: Option<String>,
    pub year: Option<Uint64>,
}
