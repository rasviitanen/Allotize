use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::net_traits::VersionedComponent;

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct RtcMetadata {
    pub connecting: usize,
    pub open: usize,
    pub closing: usize,
    pub closed: usize,
}

#[wasm_bindgen]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct PoolMetadata {
    pub rtc: RtcMetadata,
}

/// A message sent over a datachannel
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum RtcCommand {
    Share,
    Put,
    CrdtPut,
    Remove,
    Done,
}

/// A message sent over a datachannel
#[derive(Serialize, Deserialize, Debug)]
pub struct RtcMessage {
    pub command: RtcCommand,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<VersionedComponent>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Protocol {
    OneToAll,
    OneToOne,
    OneToRoom,
    OneToSelf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum SignalingAction {
    Answer,
    Candidate,
    HandleConnection,
    Heartbeat,
    Offer,
    ReconnectOffer,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignalingMessage {
    pub action: SignalingAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    pub from: String,
    pub protocol: Protocol,
    pub room: String,
}

/// There is no good way of serializing `RtcIceCandidate`, this is used in-place
#[derive(Debug, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_mid: Option<String>,
}

/// Status indicator for `PeerConnection`
#[derive(Debug, PartialEq)]
pub enum PeerConnectionStatus {
    Open,
    Closed,
    Connecting,
}

impl std::fmt::Display for PeerConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PeerConnectionStatus::{:?}", self)
    }
}
