use crate::messages::*;
use warp_ws_server::{WSMsgOut, BoxError, publish, sub, unsub, sub_all, GetEz};
use crate::{db, WSConnections_};
use uuid::Uuid;
#[macro_use]
use diesel_utils::*;
use crate::schema::{self,*};
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;
use crate::types::{leagues::*, users::*, drafts::*, fantasy_teams::*};
use crate::subscriptions::SubType;
use crate::drafting;
use crate::errors;
use std::collections::HashMap;
use tokio::sync::{MutexGuard, Mutex};
use std::sync::Arc;
use tokio::runtime::Runtime;


pub async fn insert_leagues(method: &str, message_id: Uuid, data: Vec<League>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    // TODO reduce the ridiculousness of the Values type
    //let leagues: Vec<League> = db::insert::<League, leagues::table, diesel::insertable::OwnedBatchInsert<diesel::query_builder::ValuesClause<(_, _, _, _, _, _, _, _, _), schema::leagues::table>, schema::leagues::table>>(req, conn, leagues::table)?;
    let leagues: Vec<League> = insert!(&conn, leagues::table, data)?;
    println!("{:?}", &leagues);
    publish::<SubType, League>(
        ws_conns, &leagues, SubType::League, None
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, leagues);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_leagues(method: &str, message_id: Uuid, data: Vec<LeagueUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let leagues: Vec<League> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, leagues, league_id, c)
    }).collect()})?;
    publish::<SubType, League>(
        ws_conns, &leagues, SubType::League, None
    ).await?;
    println!("{:?}", &leagues);
    let resp_msg = WSMsgOut::resp(message_id, method, leagues);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_periods(method: &str, message_id: Uuid, data: Vec<Period>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:#?}", &data);
    let out: Vec<Period> = insert!(&conn, periods::table, data)?;
    println!("{:#?}", &out);
    publish::<SubType, Period>(
        ws_conns, &out,  SubType::League, None
    ).await?;
    println!("postpublish");
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_periods(method: &str, message_id: Uuid, data: Vec<PeriodUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let out: Vec<Period> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, periods, period_id, c)
    }).collect()})?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    //publish_leagues(ws_conns, &leagues).await;
    publish::<SubType, Period>(
        ws_conns, &out,  SubType::League, None
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_stat_multipliers(method: &str, message_id: Uuid, data: Vec<StatMultiplier>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<StatMultiplier> = insert!(&conn, stat_multipliers::table, data)?;
    publish::<SubType, StatMultiplier>(
        ws_conns, &out,  SubType::League, None
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_stat_multipliers(method: &str, message_id: Uuid, data: Vec<StatMultiplierUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let out: Vec<StatMultiplier> = conn.build_transaction().run(|| {
        data.into_iter().map(|c| {
        // TODO using 2pkey, but is it legit that cannot change name once set?
        // maybe should have a uuid pkey
        // this clone a bit hacky, the macro was originally just doing UUIDs which implement copy (string name doesnt)
        update_2pkey!(&conn, stat_multipliers, league_id, name, c.clone())
    }).collect()})?;
    publish::<SubType, StatMultiplier>(
        ws_conns, &out,  SubType::League, None
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_external_users(method: &str, message_id: Uuid, data: Vec<ExternalUser>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    // TODO reduce the ridiculousness of the Values type
    //let external_users: Vec<League> = db::insert::<League, external_users::table, diesel::insertable::OwnedBatchInsert<diesel::query_builder::ValuesClause<(_, _, _, _, _, _, _, _, _), schema::external_users::table>, schema::external_users::table>>(req, conn, external_users::table)?;
    let out: Vec<ExternalUser> = insert!(&conn, external_users::table, data)?;
    println!("{:?}", &out);
    // TODO external user publishing
    publish::<SubType, ExternalUser>(
        ws_conns, &out, SubType::User, None
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_external_users(method: &str, message_id: Uuid, data: Vec<ExternalUserUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let out: Vec<ExternalUser> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, external_users, external_user_id, c)
    }).collect()})?;
    publish::<SubType, ExternalUser>(
        ws_conns, &out, SubType::User, None
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// TODO this should really be upsert
pub async fn insert_draft_queues(method: &str, message_id: Uuid, data: Vec<DraftQueue>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<DraftQueue> = insert!(&conn, draft_queues::table, data)?;
    // TODO do draft-queues even want publishing to anyone except caller (person's queue should be private)
    //let id_map = db::get_league_ids_for_draft_queues(&conn, &series_ids)?;
    // publish_for_leagues::<DraftQueue>(
    //     ws_conns, &out,
    //     out.iter().map(|c| (c.draft, c.league_id)).collect()
    // ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_draft_queues(method: &str, message_id: Uuid, data: Vec<DraftQueue>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let out: Vec<DraftQueue> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, draft_queues, fantasy_team_id, c)
    }).collect()})?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// Deliberate no insert_draft_choice as system creates
// We just update when pick has been made
// TODO hmmm shouldnt draft-queue also be system-generated?
//actually remove this? draft-choice should be updated by pick
pub async fn update_draft_choices(method: &str, message_id: Uuid, data: Vec<DraftChoiceUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let out: Vec<DraftChoice> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, draft_choices, draft_choice_id, c)
    }).collect()})?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_picks(method: &str, message_id: Uuid, data: Vec<Pick>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Pick> = insert!(&conn, picks::table, &data)?;
    let draft_id_map: HashMap<Uuid, Uuid> = db::get_draft_ids_for_picks(&conn, &data.iter().map(|p|p.pick_id).collect())?.into_iter().collect();
    publish::<SubType, Pick>(ws_conns, &out, SubType::Draft, Some(draft_id_map)).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_picks(method: &str, message_id: Uuid, data: Vec<PickUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let out: Vec<Pick> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, picks, pick_id, c)
    }).collect()})?;
    let draft_id_map: HashMap<Uuid, Uuid> = db::get_draft_ids_for_picks(&conn, &data.iter().map(|p|p.pick_id).collect())?.into_iter().collect();
    publish::<SubType, Pick>(ws_conns, &out, SubType::Draft, Some(draft_id_map)).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// async fn inner<'a>(
//     conn: &PgConn,
//     data: &'a Vec<ActivePick>,
//     player_position_cache_mut: Arc<Mutex<Option<HashMap<Uuid, String>>>>, 
//     player_team_cache_mut: Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>
// ) -> Result<(&'a HashMap<Uuid, String>, &'a HashMap<Uuid, Uuid>, Vec<db::VecUuid>, &'a League), BoxError>{
//     let _ = db::upsert_active_picks(&conn, &data)?;
//     let pick_ids = &data.iter().map(|ap|ap.pick_id).collect();
//     let all_teams = db::get_all_updated_teams(&conn, pick_ids)?;
//     let leagues = db::get_leagues_for_picks(&conn, pick_ids)?;
//     if leagues.len() > 1{
//         return Err(Box::new(errors::InvalidInputError{description: "Active picks specified are from more than one league"}) as BoxError)
//     }
//     let league = match leagues.first(){
//         Some(league) => league,
//         None => {return Err(Box::new(errors::InvalidInputError{description: "Could not find a league for active picks"}) as BoxError)}
//     };
//     let player_position_cache_opt = player_position_cache_mut.lock().await;
//     let player_team_cache_opt = player_team_cache_mut.lock().await;
//     match (player_position_cache_opt.as_ref(), player_team_cache_opt.as_ref()){
//         (Some(ref player_position_cache), Some(ref player_team_cache)) => {
//             Ok((player_position_cache, player_team_cache, all_teams, league))
//         },
//         _ => {Err(Box::new(errors::CustomError{description: "Player team and position caches not yet populated"}) as BoxError)}
//     }
// }

async fn get_cache_mutexs<'a>(
    player_position_cache_mut: &'a Arc<Mutex<Option<HashMap<Uuid, String>>>>, 
    player_team_cache_mut: &'a Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>
) -> (MutexGuard<'a, Option<HashMap<Uuid, String>>>, MutexGuard<'a, Option<HashMap<Uuid, Uuid>>>){
    let player_position_cache_opt = player_position_cache_mut.lock().await;
    let player_team_cache_opt = player_team_cache_mut.lock().await;
    (player_position_cache_opt, player_team_cache_opt)
}

pub async fn upsert_active_picks(
    method: &str, message_id: Uuid, data: Vec<ActivePick>, conn: PgConn, ws_conns: &mut WSConnections_,
    player_position_cache_mut: Arc<Mutex<Option<HashMap<Uuid, String>>>>, 
    player_team_cache_mut: Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>,
) -> Result<String, BoxError>{
    // ideas: just do the sql, then rollback if invalid (it should rollback on error)
    println!("{:?}", &data);
    // TODO How to await inside the transaction????
    // Really shouldnt lock these for so long, means can only do one pick-update at a time
    // let player_position_cache_opt = player_position_cache_mut.lock().await;
    // let player_team_cache_opt = player_team_cache_mut.lock().await;
    conn.build_transaction().run(|| {
        // let (player_position_cache, player_team_cache, all_teams, league) = inner(
        //     conn, data, player_position_cache_mut, player_team_cache_mut
        // ).await?;
        let _ = db::upsert_active_picks(&conn, &data)?;
        let pick_ids = &data.iter().map(|ap|ap.pick_id).collect();
        let all_teams = db::get_all_updated_teams(&conn, pick_ids)?;
        let leagues = db::get_leagues_for_picks(&conn, pick_ids)?;
        if leagues.len() > 1{
            return Err(Box::new(errors::InvalidInputError{description: "Active picks specified are from more than one league"}) as BoxError)
        }
        let league = match leagues.first(){
            Some(league) => league,
            None => {return Err(Box::new(errors::InvalidInputError{description: "Could not find a league for active picks"}) as BoxError)}
        };
        // let player_position_cache_opt = player_position_cache_mut.lock().await;
        // let player_team_cache_opt = player_team_cache_mut.lock().await;
        // https://stackoverflow.com/a/52521592/3920439
        // This essentially forces an async func, into a synchronous context.
        // Diesel doesnt support async in transactions yet.
        let (player_position_cache_opt, player_team_cache_opt) = Runtime::new().unwrap().block_on(
            get_cache_mutexs(&player_position_cache_mut, &player_team_cache_mut)
        );
        match (player_position_cache_opt.as_ref(), player_team_cache_opt.as_ref()){
            (Some(ref player_position_cache), Some(ref player_team_cache)) => {
                let verified_teams = drafting::verify_teams(
                    all_teams, player_position_cache,
                    player_team_cache,
                    &league.max_team_players_same_team,
                    &league.max_team_players_same_position,
                    &league.team_size
                );
                Ok(verified_teams)
            },
            _ => {Err(Box::new(errors::CustomError{description: "Player team and position caches not yet populated"}) as BoxError)}
        }?
    })?;
    let draft_id_map: HashMap<Uuid, Uuid> = db::get_draft_ids_for_picks(&conn, &data.iter().map(|p|p.pick_id).collect())?.into_iter().collect();
    publish::<SubType, ActivePick>(ws_conns, &data, SubType::Draft, Some(draft_id_map)).await?;

    // publish::<SubType, FantasyTeam>(
    //     ws_conns, &out, SubType::User, None
    // ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())

    // conn.build_transaction().run(|| {
    //     let upserted = db::upsert_active_picks(&conn, &data);
    //     let pick_ids = data.iter().map(|ap|ap.pick_id).collect();
    //     let all_teams = db::get_all_updated_teams(&conn, pick_ids)?;
    //     let leagues = db::get_leagues_for_picks(&conn, pick_ids)?;
    //     if leagues.len() > 1{
    //         return Err(Box::new(errors::InvalidInputError{description: "Active picks specified are from more than one league"}) as BoxError)
    //     }
    //     let league = match leagues.first(){
    //         Some(league) => league,
    //         None => {return Err(Box::new(errors::InvalidInputError{description: "Could not find a league for active picks"}) as BoxError)}
    //     };
    //     let player_position_cache_opt = player_position_cache_mut.lock().await;
    //     let player_team_cache_opt = player_team_cache_mut.lock().await;
    //     match (*player_position_cache_opt, *player_team_cache_opt){
    //         (Some(ref player_position_cache), Some(ref player_team_cache)) => {
    //             let verified_teams = drafting::verify_teams(
    //                 all_teams, player_position_cache,
    //                 player_team_cache,
    //                 &league.max_team_players_same_team,
    //                 &league.max_team_players_same_position,
    //                 &league.team_size);
    //                 Ok(verified_teams)
    //             },
    //         _ => {Err(Box::new(errors::CustomError{description: "Player team and position caches not yet populated"}) as BoxError)}
    //     }?
    // });
}

pub async fn insert_fantasy_teams(method: &str, message_id: Uuid, data: Vec<FantasyTeam>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<FantasyTeam> = insert!(&conn, fantasy_teams::table, data)?;
    // TODO also publish for user?
    publish::<SubType, FantasyTeam>(
        ws_conns, &out, SubType::User, None
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_fantasy_teams(method: &str, message_id: Uuid, data: Vec<FantasyTeamUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    println!("{:?}", &data);
    let out: Vec<FantasyTeam> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, fantasy_teams, fantasy_team_id, c)
    }).collect()})?;
    publish::<SubType, FantasyTeam>(
        ws_conns, &out, SubType::User, None
    ).await?;
    // TODO what's the subscription for this?
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// Could prob commonise the sub-methods into ws-server
pub async fn sub_leagues(method: &str, message_id: Uuid, data: SubLeague, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    // let data: ApiSubLeagues = serde_json::from_value(data).map_err(|e: serde_json::error::Error|
    //      serde::ser::Error::custom(format!("{}. line: {}, column: {}", e.to_string(), e.line(), e.column())
    //     ))?;
    // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = data.all{
        sub_all(&SubType::League, ws_user, toggle).await;
    }
    if let Some(ids) = data.sub_league_ids{
        sub(&SubType::League, ws_user, ids.iter()).await;
    }
    if let Some(ids) = data.unsub_league_ids{
        unsub(&SubType::League, ws_user, ids.iter()).await;
    }
    let subscription = ws_user.subscriptions.get_ez(&SubType::League);
    let data = match subscription.all{
        true => {
            db::get_full_leagues(&conn, None)
        },
        false => {
            db::get_full_leagues(&conn, Some(subscription.ids.iter().collect()))
        }
    }?;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_drafts(method: &str, message_id: Uuid, data: SubDraft, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = data.all{
        sub_all(&SubType::Draft, ws_user, toggle).await;
    }
    if let Some(ids) = data.sub_draft_ids{
        sub(&SubType::Draft, ws_user, ids.iter()).await;
    }
    if let Some(ids) = data.unsub_draft_ids{
        unsub(&SubType::Draft, ws_user, ids.iter()).await;
    }
    let subscription = ws_user.subscriptions.get_ez(&SubType::Draft);
    let data = match subscription.all{
        true => {
            db::get_full_drafts(&conn, None)
        },
        false => {
            db::get_full_drafts(&conn, Some(subscription.ids.iter().collect()))
        }
    }?;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_external_users(method: &str, message_id: Uuid, data: SubUser, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    println!("{:?}", &data);
    sub_all(&SubType::User, ws_user, data.toggle).await;
    match data.toggle{
        true => {
            let t: (Vec<ExternalUser>, Vec<Commissioner>) = db::get_users(&conn)?;
            let data = UsersAndCommissioners{users: t.0, commissioners: t.1};
            let resp_msg = WSMsgOut::resp(message_id, method, data);
            serde_json::to_string(&resp_msg).map_err(|e| e.into())
        },
        false => {
            let data = serde_json::json!({});
            let resp_msg = WSMsgOut::resp(message_id, method, data);
            serde_json::to_string(&resp_msg).map_err(|e| e.into())
        }
    }
}