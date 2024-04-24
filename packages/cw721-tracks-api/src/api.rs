use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint64;

#[cw_serde]
pub struct TrackMetadata {
    pub name: String,
    pub album_name: String,
    pub album_artwork_url: String,
    pub album_year: Uint64,
    pub track_name: String,
    pub audio_track_url: String,
}
