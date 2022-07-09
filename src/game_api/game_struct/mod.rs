use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use derive_more::{Display, Error};
use enum_map::{Enum, EnumMap};
use mongodb::bson::Bson;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::auth::{User, UserId};

#[derive(
    Debug,
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    derive_more::From,
    derive_more::Into,
)]
#[repr(transparent)]
pub struct ItemId(i64);

impl From<ItemId> for Bson {
    fn from(val: ItemId) -> Self {
        val.0.into()
    }
}

trait GameObject {
    fn get_name(&self) -> String;

    fn get_id(&self) -> ItemId;
}

#[derive(Debug, Display, Error, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameError {
    #[display(fmt = "Insufficient resource.")]
    NotEnough {
        item: String,
        amount: i64
    },
    #[display(fmt = "Illegal action.")]
    IllegalAction,
}

#[derive(Display, Copy, Clone, Hash, Enum, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Currency {
    Coins,
    Keks,
    Keys,
    Fragments,
}

impl GameObject for Currency {
    fn get_name(&self) -> String {
        format!("Currency:{}", self)
    }

    fn get_id(&self) -> ItemId {
        ItemId(0)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum PlayerEvent {
    LevelUp,
}

#[derive(Default, Clone, Copy)]
#[repr(transparent)]
pub struct CurrencyMap(EnumMap<Currency, i64>);

impl Serialize for CurrencyMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0 {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for CurrencyMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<Currency, i64> = Deserialize::deserialize(deserializer)?;
        Ok(CurrencyMap(EnumMap::from_iter(map.into_iter())))
    }
}

impl Index<Currency> for CurrencyMap {
    type Output = i64;

    fn index(&self, index: Currency) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<Currency> for CurrencyMap {
    fn index_mut(&mut self, index: Currency) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Player {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<UserId>,
    pub level: i64,
    pub xp: i64,
    pub energy: f64,
    pub currencies: CurrencyMap,
}

pub struct Inventory<'a>(&'a Player);

impl Default for Player {
    fn default() -> Self {
        Self {
            id: None,
            level: 1,
            xp: 0,
            energy: 100.0,
            currencies: Default::default(),
        }
    }
}

impl Player {
    pub fn new(user: &User) -> Self {
        Self {
            id: Some(user.id),
            ..Self::default()
        }
    }

    pub fn get_inventory(&self) -> Inventory {
        Inventory(self)
    }

    pub fn take_currency(&mut self, curr: Currency, amount: i64) -> Result<(), GameError> {
        if self.currencies.0[curr] < amount {
            return Err(GameError::NotEnough {
                item: curr.get_name(),
                amount: amount - self.currencies.0[curr],
            });
        }

        self.currencies.0[curr] -= amount;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct PlayerItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<UserId>,
    item_id: i64,
}

pub struct ActionContext {
    snapshot: Player,
    events: Vec<PlayerEvent>,
}

fn xp_for_level_up(level: i64) -> i64 {
    level * 1000
}

impl ActionContext {
    pub fn do_with(
        player: &mut Player,
        f: fn(&mut Player) -> Result<(), GameError>,
    ) -> Result<Vec<PlayerEvent>, GameError> {
        let mut ctx = Self {
            snapshot: *player,
            events: Vec::new(),
        };

        f(player)?;
        ctx.update(player);
        Ok(ctx.events)
    }

    fn update(&mut self, observed_player: &Player) {
        if observed_player.level > self.snapshot.level {
            self.events.push(PlayerEvent::LevelUp);
            self.snapshot = *observed_player;
        }
    }
}