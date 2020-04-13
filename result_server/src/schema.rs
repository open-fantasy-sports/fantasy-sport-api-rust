table! {
    competitions (competition_id) {
        competition_id -> Uuid,
        name -> Text,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

table! {
    matches (match_id) {
        match_id -> Uuid,
        name -> Text,
        series_id -> Uuid,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

table! {
    player_names (player_name_id) {
        player_name_id -> Uuid,
        player_id -> Uuid,
        name -> Text,
        timespan -> Tstzrange,
    }
}

table! {
    player_results (player_result_id) {
        player_result_id -> Uuid,
        player_id -> Uuid,
        match_id -> Uuid,
        result -> Text,
        meta -> Jsonb,
    }
}

table! {
    players (player_id) {
        player_id -> Uuid,
        meta -> Jsonb,
    }
}

table! {
    series (series_id) {
        series_id -> Uuid,
        name -> Text,
        competition_id -> Uuid,
        meta -> Jsonb,
        timespan -> Tstzrange,
    }
}

table! {
    series_teams (series_id, team_id) {
        series_id -> Uuid,
        team_id -> Uuid,
    }
}

table! {
    team_names (team_name_id) {
        team_name_id -> Uuid,
        team_id -> Uuid,
        name -> Text,
        timespan -> Tstzrange,
    }
}

table! {
    team_players (team_player_id) {
        team_player_id -> Uuid,
        player_id -> Uuid,
        team_id -> Uuid,
        timespan -> Tstzrange,
    }
}

table! {
    team_results (team_result_id) {
        team_result_id -> Uuid,
        team_id -> Uuid,
        match_id -> Uuid,
        result -> Text,
        meta -> Jsonb,
    }
}

table! {
    teams (team_id) {
        team_id -> Uuid,
        meta -> Jsonb,
    }
}

table! {
    team_series_results (team_series_result_id) {
        team_series_result_id -> Uuid,
        team_id -> Uuid,
        series_id -> Uuid,
        result -> Text,
        meta -> Jsonb,
    }
}

joinable!(matches -> series (series_id));
joinable!(player_names -> players (player_id));
joinable!(player_results -> matches (match_id));
joinable!(player_results -> players (player_id));
joinable!(series -> competitions (competition_id));
joinable!(series_teams -> series (series_id));
joinable!(series_teams -> teams (team_id));
joinable!(team_names -> teams (team_id));
joinable!(team_players -> players (player_id));
joinable!(team_players -> teams (team_id));
joinable!(team_results -> matches (match_id));
joinable!(team_results -> teams (team_id));
joinable!(team_series_results -> series (series_id));
joinable!(team_series_results -> teams (team_id));

allow_tables_to_appear_in_same_query!(
    competitions,
    matches,
    player_names,
    player_results,
    players,
    series,
    series_teams,
    team_names,
    team_players,
    team_results,
    teams,
    team_series_results,
);
