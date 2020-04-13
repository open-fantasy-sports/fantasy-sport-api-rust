-- Your SQL goes here
-- technically possible to dynamically have one func for all required tables
-- however not sure it's worth the faff
-- https://stackoverflow.com/questions/10705616/table-name-as-a-postgresql-function-parameter
CREATE FUNCTION public.trim_team_name_timespans(new_team_id uuid, new_timespan tstzrange) 
	RETURNS void
	LANGUAGE plpgsql
AS $function$
DECLARE
	existing_row RECORD;
BEGIN
	FOR existing_row IN
		SELECT team_name_id, timespan FROM team_names
		WHERE team_id = new_team_id AND timespan && new_timespan
	LOOP
		IF new_timespan @> existing_row.timespan THEN
			DELETE FROM team_names WHERE team_name_id = existing_row.team_name_id;
		ELSIF new_timespan @> lower(existing_row.timespan) THEN
			UPDATE team_names
			SET timespan = tstzrange(upper(new_timespan), upper(existing_row.timespan), "[)")
			WHERE team_name_id = existing_row.team_name_id;
		ELSIF new_timespan @> upper(existing_row.timespan) THEN
			UPDATE team_names
			SET timespan = tstzrange(lower(existing_row.timespan), lower(new_timespan), "[)")
			WHERE team_name_id = existing_row.team_name_id;
		END IF;
	END LOOP;
END $function$;

CREATE FUNCTION public.trim_player_name_timespans(new_player_id uuid, new_timespan tstzrange) 
	RETURNS void
	LANGUAGE plpgsql
AS $function$
DECLARE
	existing_row RECORD;
BEGIN
	FOR existing_row IN
		SELECT player_name_id, timespan FROM player_names
		WHERE player_id = new_player_id AND timespan && new_timespan
	LOOP
		IF new_timespan @> existing_row.timespan THEN
			DELETE FROM player_names WHERE player_name_id = existing_row.player_name_id;
		ELSIF new_timespan @> lower(existing_row.timespan) THEN
			UPDATE player_names
			SET timespan = tstzrange(upper(new_timespan), upper(existing_row.timespan), "[)")
			WHERE player_name_id = existing_row.player_name_id;
		ELSIF new_timespan @> upper(existing_row.timespan) THEN
			UPDATE player_names
			SET timespan = tstzrange(lower(existing_row.timespan), lower(new_timespan), "[)")
			WHERE player_name_id = existing_row.player_name_id;
		END IF;
	END LOOP;
END $function$;

CREATE FUNCTION public.trim_team_player_timespans(new_player_id uuid, new_timespan tstzrange) 
	RETURNS void
	LANGUAGE plpgsql
AS $function$
DECLARE
	existing_row RECORD;
BEGIN
	FOR existing_row IN
		SELECT player_id, timespan FROM team_players
		WHERE player_id = new_player_id AND timespan && new_timespan
	LOOP
		IF new_timespan @> existing_row.timespan THEN
			DELETE FROM team_players WHERE team_player_id = existing_row.team_player_id;
		ELSIF new_timespan @> lower(existing_row.timespan) THEN
			UPDATE team_players
			SET timespan = tstzrange(upper(new_timespan), upper(existing_row.timespan), "[)")
			WHERE team_player_id = existing_row.team_player_id;
		ELSIF new_timespan @> upper(existing_row.timespan) THEN
			UPDATE team_players
			SET timespan = tstzrange(lower(existing_row.timespan), lower(new_timespan), "[)")
			WHERE team_player_id = existing_row.team_player_id;
		END IF;
	END LOOP;
END $function$;
