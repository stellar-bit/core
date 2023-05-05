use crate::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "type", content = "content")]
pub enum ServerResponse {
    SyncGameCmds(Vec<(User, GameCmd)>),
    SyncFullGame(Game),
    SetUser(User),
    SlowDown,
    Success,
    Error,
    SyncClock(time::Duration),
}

#[derive(Serialize, Deserialize, Debug)]
// #[serde(tag = "type", content = "content")]
pub enum ClientRequest {
    ExecuteGameCmds(Vec<GameCmd>), // "{\"ExecGameCmds\":[{\"SpawnAsteroid\":[193.66406,126.02344]}, {\"ExecuteComponentCmd\":[0, 10, 2, {"SetActive": true}}]
    Join(PlayerToken, PlayerToken),
    FullGameSync,
    GameCmdsSync,
    SyncClock,
}

#[derive(Debug)]
pub enum NetworkError {
    IncorrectDataFormat,
    WebsocketTrouble,
    NoMsgReceived,
    WrongAuthToken,
    NotAuthorized,
}
