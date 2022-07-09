use actix::fut::{ok, Ready};
use actix_web::dev::Payload;
use actix_web::error::ErrorInternalServerError;
use actix_web::{web, Error, FromRequest, HttpRequest};
use log::error;

use crate::auth::{User, UserId};
use mongodb::bson::{doc, Document};
use mongodb::options::{Acknowledgment, ReadConcern, TransactionOptions, WriteConcern};
use mongodb::{Client, ClientSession, Collection, Database};

use crate::game_api::game_struct::Player;

pub struct UserData {
    pub client: web::Data<Client>,
    pub db: Database,
    pub players: Collection<Player>,
}

impl UserData {
    pub async fn get_player(&self, id: UserId) -> Result<Option<Player>, Error> {
        self.players
            .find_one(filter_by_user_id(id), None)
            .await
            .map_err(|e| {
                error!("{:?}", e);
                ErrorInternalServerError("Internal server error.")
            })
    }

    pub async fn get_player_or_create(&self, user: &User) -> Result<Player, Error> {
        self.get_player(user.id)
            .await
            .map(|u| u.unwrap_or_else(|| Player::new(user)))
    }

    pub async fn get_player_session(
        &self,
        id: UserId,
        session: &mut ClientSession,
    ) -> Result<Option<Player>, Error> {
        self.players
            .find_one_with_session(filter_by_user_id(id), None, session)
            .await
            .map_err(|e| {
                error!("{:?}", e);
                ErrorInternalServerError("Internal server error.")
            })
    }

    pub async fn get_player_session_or_create(
        &self,
        user: &User,
        session: &mut ClientSession,
    ) -> Result<Player, Error> {
        self.get_player_session(user.id, session)
            .await
            .map(|u| u.unwrap_or_else(|| Player::new(user)))
    }

    pub async fn start_session(&self) -> Result<ClientSession, Error> {
        let mut session = self.client.start_session(None).await.map_err(|e| {
            error!("{:?}", e);
            ErrorInternalServerError("Internal server error.")
        })?;

        let options = TransactionOptions::builder()
            .read_concern(ReadConcern::majority())
            .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
            .build();

        session
            .start_transaction(options)
            .await
            .map_err(|_| ErrorInternalServerError("Internal server error."))?;

        Ok(session)
    }
}

pub fn filter_by_user(user: &User) -> Document {
    filter_by_user_id(user.id)
}

pub fn filter_by_user_id(id: UserId) -> Document {
    doc! {
        "_id": id
    }
}

impl FromRequest for UserData {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let client = req.app_data::<web::Data<Client>>().unwrap();
        let db = client.database("game");
        ok(UserData {
            client: client.clone(),
            db: db.clone(),
            players: db.collection::<Player>("players"),
        })
    }
}
