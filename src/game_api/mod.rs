use actix_web::{get, post, web, Responder};
use mongodb::options::ReplaceOptions;

use crate::auth::User;
use crate::game_api::db::filter_by_user;
use crate::game_api::game_err::CommandResult;
use crate::game_api::game_struct::{ActionContext, Currency};

mod db;
mod game_err;
mod commands;
pub mod game_struct;

#[get("/balance")]
async fn balance(user: User, db: db::UserData) -> CommandResult<impl Responder> {
    let player = db.get_player_or_create(&user).await?;
    Ok(web::Json(player))
}

#[post("/spend")]
async fn spend(user: User, db: db::UserData) -> CommandResult<impl Responder> {
    let mut session = db.start_session().await?;
    let mut player = db.get_player_session_or_create(&user, &mut session).await?;
    let events = ActionContext::do_with(&mut player, |plr| {
        plr.take_currency(Currency::Coins, 150)?;
        Ok(())
    })?;

    let options: ReplaceOptions = ReplaceOptions::builder().upsert(true).build();
    db.players
        .replace_one_with_session(filter_by_user(&user), &player, Some(options), &mut session)
        .await?;

    session.commit_transaction().await?;

    Ok(web::Json(events))
}

pub fn define_services(cfg: &mut web::ServiceConfig) {
    cfg.service(balance);
    cfg.service(commands::daily::daily);
    cfg.service(spend);
}
