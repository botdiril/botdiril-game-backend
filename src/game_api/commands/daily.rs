use actix_web::{post, Responder, web};
use mongodb::options::ReplaceOptions;

use crate::{game_api::{game_err::CommandResult, db::{self, filter_by_user}, game_struct::{ActionContext, Currency}}, auth::User};

#[post("/daily")]
async fn daily(user: User, db: db::UserData) -> CommandResult<impl Responder> {
    let mut session = db.start_session().await?;
    let mut player = db.get_player_session_or_create(&user, &mut session).await?;
    let events = ActionContext::do_with(&mut player, |plr| {
        plr.currencies[Currency::Coins] += 150;
        Ok(())
    })?;

    let mut options = ReplaceOptions::default();
    options.upsert = Some(true);

    db.players
        .replace_one_with_session(filter_by_user(&user), &player, Some(options), &mut session)
        .await?;

    session.commit_transaction().await?;

    Ok(web::Json(events))
}