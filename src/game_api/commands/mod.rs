use serde::Serialize;
use super::game_struct::{PlayerEvent};

pub(super) mod daily;

#[derive(Serialize, Clone, Default)]
struct CommandOutput<D: Serialize + Default> {
    data: D,
    events: Vec<PlayerEvent>,
}