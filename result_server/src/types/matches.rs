use serde::{Deserialize, Serialize};
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use frunk::labelled::transform_from;
use super::series::Series;
use super::results::{TeamMatchResult, PlayerResult};
use crate::publisher::Publishable;
use diesel_utils::{PgConn, DieselTimespan, my_timespan_format, my_timespan_format_opt};
use itertools::Itertools;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros


#[derive(Insertable, Deserialize, LabelledGeneric, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(Series)]
#[primary_key(match_id)]
#[table_name = "matches"]
pub struct Match {
    pub match_id: Uuid,
    pub name: String,
    pub series_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, LabelledGeneric, Debug, AsChangeset)]
#[primary_key(match_id)]
#[table_name = "matches"]
pub struct MatchUpdate {
    pub match_id: Uuid,
    pub series_id: Option<Uuid>,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
}

#[derive(Deserialize, Serialize, Debug, Clone, LabelledGeneric)]
pub struct ApiMatch{
    pub match_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub player_results: Vec<PlayerResult>,
    pub team_results: Vec<TeamMatchResult>
}

#[derive(Deserialize, Serialize, Debug, Clone, LabelledGeneric)]
pub struct ApiMatchNew{
    pub match_id: Uuid,
    pub series_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub player_results: Vec<PlayerResult>,
    pub team_results: Vec<TeamMatchResult>
}

impl ApiMatch{
    pub fn insertable(self, series_id: Uuid) -> (Match, Vec<PlayerResult>, Vec<TeamMatchResult>){
        (
            Match{match_id: self.match_id, name: self.name, meta: self.meta, timespan: self.timespan, series_id},
            self.player_results,
            self.team_results
        )
    }
}

impl ApiMatchNew{
    pub fn insert(conn: &PgConn, new: Vec<ApiMatchNew>) -> Result<bool, diesel::result::Error>{
        let (mut player_results, mut team_match_results) = (vec![], vec![]);
        let matches: Vec<Match> = new
            .into_iter().map(|m|{
                let series_id = m.series_id;
                let m2: ApiMatch = transform_from(m);
                let mut tup = m2.insertable(series_id);
                player_results.append(&mut tup.1);
                team_match_results.append(&mut tup.2);
                tup.0
            }).collect_vec();
            insert_exec!(conn, matches::table, matches)?;
            insert_exec!(conn, player_results::table, player_results)?;
            insert_exec!(conn, team_match_results::table, team_match_results)?;
            Ok(true)
    }
}

impl Publishable for Match {
    fn message_type<'a>() -> &'a str {
        "matches_update"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.match_id
    }
}

impl Publishable for ApiMatchNew {
    fn message_type<'a>() -> &'a str {
        "matches"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.match_id
    }
}

// use crate::handlers::ServerInsertable;

// #[async_trait]
// impl ServerInsertable for ApiMatchNew{
//     fn comp_id_map_tup(
//         conn: PgConn,
//         me: &Vec<Self>,
//     ) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error>{
//         let series_ids: Vec<Uuid> = me.iter().map(|s| s.series_id).dedup().collect();
//         db::get_competition_ids_for_series(&conn, &series_ids)
//     }

// }