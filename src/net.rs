use serde::{Deserialize, Serialize};

use crate::board;


#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    PlayerJoin(PlayerJoinMessage),
    PlayerLeave(PlayerLeaveMessage),

    BoardState(BoardStateMessage),
    StartBid(StartBidMessage),
    UpdateBid(UpdateBidMessage),
    MakeBid(MakeBidMessage),
    StartEval(StartEvalMessage)
}

/// Sent when a player joins.
/// 
/// Host -> All Clients
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerJoinMessage {
    pub id: String,
    pub name: String,
}

/// Sent when a player leaves.
/// 
/// Host -> All Clients
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlayerLeaveMessage {
    pub id: String,
}

/// Sent when the board state changes.
/// 
/// Host -> All Clients
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BoardStateMessage {
    pub board: board::Board,
}

/// Sent when the bidding
/// phase begins.
/// 
/// Host -> All Clients
#[derive(Clone, Serialize, Deserialize)]
pub struct StartBidMessage {
    pub end_time: u64,
}

/// Sent when a new bid is made.
/// 
/// Client -> Host
#[derive(Clone, Serialize, Deserialize)]
pub struct MakeBidMessage {
    pub bid: u8,
}

/// Sent when a new bid is made.
/// 
/// Host -> All Clients
#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateBidMessage {
    /// The ID of the player
    pub player: String,
    pub bid: u8,
}

/// Sent when a player's bid is up for evaluation.
/// 
/// Host -> All Clients
#[derive(Clone, Serialize, Deserialize)]
pub struct StartEvalMessage {
    pub player: String,
}


