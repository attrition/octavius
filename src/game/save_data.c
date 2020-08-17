#include "save_data.h"

#include "building/barracks.h"
#include "building/count.h"
#include "building/list.h"
#include "building/storage.h"
#include "city/culture.h"
#include "city/data.h"
#include "city/message.h"
#include "city/view.h"
#include "core/dir.h"
#include "core/random.h"
#include "empire/city.h"
#include "empire/empire.h"
#include "empire/trade_prices.h"
#include "empire/trade_route.h"
#include "figure/enemy_army.h"
#include "figure/formation.h"
#include "figure/name.h"
#include "figure/route.h"
#include "figure/trader.h"
#include "game/migrate_save_data.h"
#include "game/save_data.h"
#include "game/time.h"
#include "game/tutorial.h"
#include "map/aqueduct.h"
#include "map/bookmark.h"
#include "map/building.h"
#include "map/desirability.h"
#include "map/elevation.h"
#include "map/figure.h"
#include "map/image.h"
#include "map/property.h"
#include "map/random.h"
#include "map/routing.h"
#include "map/sprite.h"
#include "map/terrain.h"
#include "scenario/criteria.h"
#include "scenario/earthquake.h"
#include "scenario/emperor_change.h"
#include "scenario/gladiator_revolt.h"
#include "scenario/invasion.h"
#include "scenario/map.h"
#include "scenario/scenario.h"
#include "sound/city.h"

#include <stdlib.h>
#include <string.h>

void init_file_piece(file_piece *piece, int size, int compressed)
{
    piece->compressed = compressed;
    void *data = malloc(size);
    memset(data, 0, size);
    buffer_init(&piece->buf, data, size);
}

buffer *create_scenario_piece(scenario_data *data, int size)
{
    file_piece *piece = &data->pieces[data->num_pieces++];
    init_file_piece(piece, size, 0);
    return &piece->buf;
}

buffer *create_savegame_piece(savegame_data *data, int size, int compressed)
{
    file_piece *piece = &data->pieces[data->num_pieces++];
    init_file_piece(piece, size, compressed);
    return &piece->buf;
}

void save_data_scenario_init_data(scenario_data *data, int version)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }

    int grid_u8 = GRID_SIZE * GRID_SIZE * sizeof(uint8_t);
    switch (version) {
        case SCENARIO_VERSION_LEGACY:
            grid_u8 = 162 * 162 * sizeof(uint8_t);
            break;
    }

    scenario_state *state = &data->state;
    
    // legacy doesn't have a scenario version
    if (version != SCENARIO_VERSION_LEGACY) {
        state->file_version = create_scenario_piece(data, 4);
    }
    state->graphic_ids = create_scenario_piece(data, grid_u8 * 2);
    state->edge = create_scenario_piece(data, grid_u8);
    state->terrain = create_scenario_piece(data, grid_u8 * 2);
    state->bitfields = create_scenario_piece(data, grid_u8);
    state->random = create_scenario_piece(data, grid_u8);
    state->elevation = create_scenario_piece(data, grid_u8);
    state->random_iv = create_scenario_piece(data, 8);
    state->camera = create_scenario_piece(data, 8);
    state->scenario = create_scenario_piece(data, 1720);
    state->end_marker = create_scenario_piece(data, 4);
}

void save_data_savegame_init_data_legacy(savegame_data *data)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }
    savegame_state *state = &data->state;
    state->scenario_campaign_mission = create_savegame_piece(data, 4, 0);
    state->file_version = create_savegame_piece(data, 4, 0);
    state->image_grid = create_savegame_piece(data, 52488, 1);
    state->edge_grid = create_savegame_piece(data, 26244, 1);
    state->building_grid = create_savegame_piece(data, 52488, 1);
    state->terrain_grid = create_savegame_piece(data, 52488, 1);
    state->aqueduct_grid = create_savegame_piece(data, 26244, 1);
    state->figure_grid = create_savegame_piece(data, 52488, 1);
    state->bitfields_grid = create_savegame_piece(data, 26244, 1);
    state->sprite_grid = create_savegame_piece(data, 26244, 1);
    state->random_grid = create_savegame_piece(data, 26244, 0);
    state->desirability_grid = create_savegame_piece(data, 26244, 1);
    state->elevation_grid = create_savegame_piece(data, 26244, 1);
    state->building_damage_grid = create_savegame_piece(data, 26244, 1);
    state->aqueduct_backup_grid = create_savegame_piece(data, 26244, 1);
    state->sprite_backup_grid = create_savegame_piece(data, 26244, 1);
    state->figures = create_savegame_piece(data, 128000, 1);
    state->route_figures = create_savegame_piece(data, 1200, 1);
    state->route_paths = create_savegame_piece(data, 300000, 1);
    state->formations = create_savegame_piece(data, 6400, 1);
    state->formation_totals = create_savegame_piece(data, 12, 0);
    state->city_data = create_savegame_piece(data, 36136, 1);
    state->city_faction_unknown = create_savegame_piece(data, 2, 0);
    state->player_name = create_savegame_piece(data, 64, 0);
    state->city_faction = create_savegame_piece(data, 4, 0);
    state->buildings = create_savegame_piece(data, 256000, 1);
    state->city_view_orientation = create_savegame_piece(data, 4, 0);
    state->game_time = create_savegame_piece(data, 20, 0);
    state->building_extra_highest_id_ever = create_savegame_piece(data, 8, 0);
    state->random_iv = create_savegame_piece(data, 8, 0);
    state->city_view_camera = create_savegame_piece(data, 8, 0);
    state->building_count_culture1 = create_savegame_piece(data, 132, 0);
    state->city_graph_order = create_savegame_piece(data, 8, 0);
    state->emperor_change_time = create_savegame_piece(data, 8, 0);
    state->empire = create_savegame_piece(data, 12, 0);
    state->empire_cities = create_savegame_piece(data, 2706, 1);
    state->building_count_industry = create_savegame_piece(data, 128, 0);
    state->trade_prices = create_savegame_piece(data, 128, 0);
    state->figure_names = create_savegame_piece(data, 84, 0);
    state->culture_coverage = create_savegame_piece(data, 60, 0);
    state->scenario = create_savegame_piece(data, 1720, 0);
    state->max_game_year = create_savegame_piece(data, 4, 0);
    state->earthquake = create_savegame_piece(data, 60, 0);
    state->emperor_change_state = create_savegame_piece(data, 4, 0);
    state->messages = create_savegame_piece(data, 16000, 1);
    state->message_extra = create_savegame_piece(data, 12, 0);
    state->population_messages = create_savegame_piece(data, 10, 0);
    state->message_counts = create_savegame_piece(data, 80, 0);
    state->message_delays = create_savegame_piece(data, 80, 0);
    state->building_list_burning_totals = create_savegame_piece(data, 8, 0);
    state->figure_sequence = create_savegame_piece(data, 4, 0);
    state->scenario_settings = create_savegame_piece(data, 12, 0);
    state->invasion_warnings = create_savegame_piece(data, 3232, 1);
    state->scenario_is_custom = create_savegame_piece(data, 4, 0);
    state->city_sounds = create_savegame_piece(data, 8960, 0);
    state->building_extra_highest_id = create_savegame_piece(data, 4, 0);
    state->figure_traders = create_savegame_piece(data, 4804, 0);
    state->building_list_burning = create_savegame_piece(data, 1000, 1);
    state->building_list_small = create_savegame_piece(data, 1000, 1);
    state->building_list_large = create_savegame_piece(data, 4000, 1);
    state->tutorial_part1 = create_savegame_piece(data, 32, 0);
    state->building_count_military = create_savegame_piece(data, 16, 0);
    state->enemy_army_totals = create_savegame_piece(data, 20, 0);
    state->building_storages = create_savegame_piece(data, 6400, 0);
    state->building_count_culture2 = create_savegame_piece(data, 32, 0);
    state->building_count_support = create_savegame_piece(data, 24, 0);
    state->tutorial_part2 = create_savegame_piece(data, 4, 0);
    state->gladiator_revolt = create_savegame_piece(data, 16, 0);
    state->trade_route_limit = create_savegame_piece(data, 1280, 1);
    state->trade_route_traded = create_savegame_piece(data, 1280, 1);
    state->building_barracks_tower_sentry = create_savegame_piece(data, 4, 0);
    state->building_extra_sequence = create_savegame_piece(data, 4, 0);
    state->routing_counters = create_savegame_piece(data, 16, 0);
    state->building_count_culture3 = create_savegame_piece(data, 40, 0);
    state->enemy_armies = create_savegame_piece(data, 900, 0);
    state->city_entry_exit_xy = create_savegame_piece(data, 16, 0);
    state->last_invasion_id = create_savegame_piece(data, 2, 0);
    state->building_extra_corrupt_houses = create_savegame_piece(data, 8, 0);
    state->scenario_name = create_savegame_piece(data, 65, 0);
    state->bookmarks = create_savegame_piece(data, 32, 0);
    state->tutorial_part3 = create_savegame_piece(data, 4, 0);
    state->city_entry_exit_grid_offset = create_savegame_piece(data, 8, 0);
    state->end_marker = create_savegame_piece(data, 284, 0); // 71x 4-bytes emptiness
}

void save_data_savegame_init_data_augustus(savegame_data *data, int version)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }

    int grid_u8 = GRID_SIZE * GRID_SIZE * sizeof(uint8_t);
    switch (version) {
        case SAVE_GAME_VERSION_AUG_V1:
            grid_u8 = 162 * 162 * sizeof(uint8_t);
            break;
    }

    savegame_state *state = &data->state;
    state->scenario_campaign_mission = create_savegame_piece(data, 4, 0);
    state->file_version = create_savegame_piece(data, 4, 0);
    state->image_grid = create_savegame_piece(data, grid_u8 * 2, 1);
    state->edge_grid = create_savegame_piece(data, grid_u8, 1);
    state->building_grid = create_savegame_piece(data, grid_u8 * 2, 1);
    state->terrain_grid = create_savegame_piece(data, grid_u8 * 2, 1);
    state->aqueduct_grid = create_savegame_piece(data, grid_u8, 1);
    state->figure_grid = create_savegame_piece(data, grid_u8 * 2, 1);
    state->bitfields_grid = create_savegame_piece(data, grid_u8, 1);
    state->sprite_grid = create_savegame_piece(data, grid_u8, 1);
    state->random_grid = create_savegame_piece(data, grid_u8, 0);
    state->desirability_grid = create_savegame_piece(data, grid_u8, 1);
    state->elevation_grid = create_savegame_piece(data, grid_u8, 1);
    state->building_damage_grid = create_savegame_piece(data, grid_u8, 1);
    state->aqueduct_backup_grid = create_savegame_piece(data, grid_u8, 1);
    state->sprite_backup_grid = create_savegame_piece(data, grid_u8, 1);
    state->figures = create_savegame_piece(data, 710000, 1);
    state->route_figures = create_savegame_piece(data, 6000, 1);
    state->route_paths = create_savegame_piece(data, 1500000, 1);
    state->formations = create_savegame_piece(data, 34500, 1);
    state->formation_totals = create_savegame_piece(data, 12, 0);
    state->city_data = create_savegame_piece(data, 36156, 1);
    state->city_faction_unknown = create_savegame_piece(data, 2, 0);
    state->player_name = create_savegame_piece(data, 64, 0);
    state->city_faction = create_savegame_piece(data, 4, 0);
    state->buildings = create_savegame_piece(data, 1280000, 1);
    state->city_view_orientation = create_savegame_piece(data, 4, 0);
    state->game_time = create_savegame_piece(data, 20, 0);
    state->building_extra_highest_id_ever = create_savegame_piece(data, 8, 0);
    state->random_iv = create_savegame_piece(data, 8, 0);
    state->city_view_camera = create_savegame_piece(data, 8, 0);
    state->building_count_culture1 = create_savegame_piece(data, 132, 0);
    state->city_graph_order = create_savegame_piece(data, 8, 0);
    state->emperor_change_time = create_savegame_piece(data, 8, 0);
    state->empire = create_savegame_piece(data, 12, 0);
    state->empire_cities = create_savegame_piece(data, 2706, 1);
    state->building_count_industry = create_savegame_piece(data, 128, 0);
    state->trade_prices = create_savegame_piece(data, 128, 0);
    state->figure_names = create_savegame_piece(data, 84, 0);
    state->culture_coverage = create_savegame_piece(data, 60, 0);
    state->scenario = create_savegame_piece(data, 1720, 0);
    state->max_game_year = create_savegame_piece(data, 4, 0);
    state->earthquake = create_savegame_piece(data, 60, 0);
    state->emperor_change_state = create_savegame_piece(data, 4, 0);
    state->messages = create_savegame_piece(data, 16000, 1);
    state->message_extra = create_savegame_piece(data, 12, 0);
    state->population_messages = create_savegame_piece(data, 10, 0);
    state->message_counts = create_savegame_piece(data, 80, 0);
    state->message_delays = create_savegame_piece(data, 80, 0);
    state->building_list_burning_totals = create_savegame_piece(data, 8, 0);
    state->figure_sequence = create_savegame_piece(data, 4, 0);
    state->scenario_settings = create_savegame_piece(data, 12, 0);
    state->invasion_warnings = create_savegame_piece(data, 3232, 1);
    state->scenario_is_custom = create_savegame_piece(data, 4, 0);
    state->city_sounds = create_savegame_piece(data, 8960, 0);
    state->building_extra_highest_id = create_savegame_piece(data, 4, 0);
    state->figure_traders = create_savegame_piece(data, 4804, 0);
    state->building_list_burning = create_savegame_piece(data, 5000, 1);
    state->building_list_small = create_savegame_piece(data, 5000, 1);
    state->building_list_large = create_savegame_piece(data, 20000, 1);
    state->tutorial_part1 = create_savegame_piece(data, 32, 0);
    state->building_count_military = create_savegame_piece(data, 16, 0);
    state->enemy_army_totals = create_savegame_piece(data, 20, 0);
    state->building_storages = create_savegame_piece(data, 32000, 0);
    state->building_count_culture2 = create_savegame_piece(data, 32, 0);
    state->building_count_support = create_savegame_piece(data, 24, 0);
    state->tutorial_part2 = create_savegame_piece(data, 4, 0);
    state->gladiator_revolt = create_savegame_piece(data, 16, 0);
    state->trade_route_limit = create_savegame_piece(data, 1280, 1);
    state->trade_route_traded = create_savegame_piece(data, 1280, 1);
    state->building_barracks_tower_sentry = create_savegame_piece(data, 4, 0);
    state->building_extra_sequence = create_savegame_piece(data, 4, 0);
    state->routing_counters = create_savegame_piece(data, 16, 0);
    state->building_count_culture3 = create_savegame_piece(data, 40, 0);
    state->enemy_armies = create_savegame_piece(data, 900, 0);
    state->city_entry_exit_xy = create_savegame_piece(data, 16, 0);
    state->last_invasion_id = create_savegame_piece(data, 2, 0);
    state->building_extra_corrupt_houses = create_savegame_piece(data, 8, 0);
    state->scenario_name = create_savegame_piece(data, 65, 0);
    state->bookmarks = create_savegame_piece(data, 32, 0);
    state->tutorial_part3 = create_savegame_piece(data, 4, 0);
    state->city_entry_exit_grid_offset = create_savegame_piece(data, 8, 0);
    state->end_marker = create_savegame_piece(data, 284, 0); // 71x 4-bytes emptiness
}

void scenario_version_save_state(buffer *buf)
{
    buffer_write_u16(buf, SCENARIO_VERSION);
}

void scenario_version_load_state(buffer *buf)
{
    buffer_skip(buf, 2);
}

void save_data_scenario_load_from_state(scenario_state *file, int version)
{
    if (version != 0) {
        scenario_version_load_state(file->file_version);
    }
    map_image_load_state(file->graphic_ids);
    map_terrain_load_state(file->terrain);
    map_property_load_state(file->bitfields, file->edge);
    map_random_load_state(file->random);
    map_elevation_load_state(file->elevation);
    city_view_load_scenario_state(file->camera);

    random_load_state(file->random_iv);

    scenario_load_state(file->scenario);

    buffer_skip(file->end_marker, 4);
}

void save_data_scenario_save_to_state(scenario_state *file)
{
    scenario_version_save_state(file->file_version);

    map_image_save_state(file->graphic_ids);
    map_terrain_save_state(file->terrain);
    map_property_save_state(file->bitfields, file->edge);
    map_random_save_state(file->random);
    map_elevation_save_state(file->elevation);
    city_view_save_scenario_state(file->camera);

    random_save_state(file->random_iv);

    scenario_save_state(file->scenario);

    buffer_skip(file->end_marker, 4);
}

void save_data_savegame_load_from_state(savegame_state *state)
{
    int savegame_version = buffer_read_i32(state->file_version);

    scenario_settings_load_state(state->scenario_campaign_mission,
        state->scenario_settings,
        state->scenario_is_custom,
        state->player_name,
        state->scenario_name);

    map_image_load_state(state->image_grid);
    map_building_load_state(state->building_grid, state->building_damage_grid);
    map_terrain_load_state(state->terrain_grid);
    map_aqueduct_load_state(state->aqueduct_grid, state->aqueduct_backup_grid);
    map_figure_load_state(state->figure_grid);
    map_sprite_load_state(state->sprite_grid, state->sprite_backup_grid);
    map_property_load_state(state->bitfields_grid, state->edge_grid);
    map_random_load_state(state->random_grid);
    map_desirability_load_state(state->desirability_grid);
    map_elevation_load_state(state->elevation_grid);

    figure_load_state(state->figures, state->figure_sequence);
    figure_route_load_state(state->route_figures, state->route_paths);
    formations_load_state(state->formations, state->formation_totals);

    city_data_load_state(state->city_data,
        state->city_faction,
        state->city_faction_unknown,
        state->city_graph_order,
        state->city_entry_exit_xy,
        state->city_entry_exit_grid_offset);

    building_load_state(state->buildings,
        state->building_extra_highest_id,
        state->building_extra_highest_id_ever,
        state->building_extra_sequence,
        state->building_extra_corrupt_houses);
    building_barracks_load_state(state->building_barracks_tower_sentry);
    city_view_load_state(state->city_view_orientation, state->city_view_camera);
    game_time_load_state(state->game_time);
    random_load_state(state->random_iv);
    building_count_load_state(state->building_count_industry,
        state->building_count_culture1,
        state->building_count_culture2,
        state->building_count_culture3,
        state->building_count_military,
        state->building_count_support);

    scenario_emperor_change_load_state(state->emperor_change_time, state->emperor_change_state);

    empire_load_state(state->empire);
    empire_city_load_state(state->empire_cities);
    trade_prices_load_state(state->trade_prices);
    figure_name_load_state(state->figure_names);
    city_culture_load_state(state->culture_coverage);

    scenario_load_state(state->scenario);
    scenario_criteria_load_state(state->max_game_year);
    scenario_earthquake_load_state(state->earthquake);
    city_message_load_state(state->messages, state->message_extra,
        state->message_counts, state->message_delays,
        state->population_messages);
    sound_city_load_state(state->city_sounds);
    traders_load_state(state->figure_traders);

    building_list_load_state(state->building_list_small, state->building_list_large,
        state->building_list_burning, state->building_list_burning_totals);

    tutorial_load_state(state->tutorial_part1, state->tutorial_part2, state->tutorial_part3);

    building_storage_load_state(state->building_storages);
    scenario_gladiator_revolt_load_state(state->gladiator_revolt);
    trade_routes_load_state(state->trade_route_limit, state->trade_route_traded);
    map_routing_load_state(state->routing_counters);
    enemy_armies_load_state(state->enemy_armies, state->enemy_army_totals);
    scenario_invasion_load_state(state->last_invasion_id, state->invasion_warnings);
    map_bookmark_load_state(state->bookmarks);

    buffer_skip(state->end_marker, 284);
}

void save_data_savegame_save_to_state(savegame_state *state, int savegame_version)
{
    buffer_write_i32(state->file_version, savegame_version);

    scenario_settings_save_state(state->scenario_campaign_mission,
        state->scenario_settings,
        state->scenario_is_custom,
        state->player_name,
        state->scenario_name);

    map_image_save_state(state->image_grid);
    map_building_save_state(state->building_grid, state->building_damage_grid);
    map_terrain_save_state(state->terrain_grid);
    map_aqueduct_save_state(state->aqueduct_grid, state->aqueduct_backup_grid);
    map_figure_save_state(state->figure_grid);
    map_sprite_save_state(state->sprite_grid, state->sprite_backup_grid);
    map_property_save_state(state->bitfields_grid, state->edge_grid);
    map_random_save_state(state->random_grid);
    map_desirability_save_state(state->desirability_grid);
    map_elevation_save_state(state->elevation_grid);

    figure_save_state(state->figures, state->figure_sequence);
    figure_route_save_state(state->route_figures, state->route_paths);
    formations_save_state(state->formations, state->formation_totals);

    city_data_save_state(state->city_data,
        state->city_faction,
        state->city_faction_unknown,
        state->city_graph_order,
        state->city_entry_exit_xy,
        state->city_entry_exit_grid_offset);

    building_save_state(state->buildings,
        state->building_extra_highest_id,
        state->building_extra_highest_id_ever,
        state->building_extra_sequence,
        state->building_extra_corrupt_houses);
    building_barracks_save_state(state->building_barracks_tower_sentry);
    city_view_save_state(state->city_view_orientation, state->city_view_camera);
    game_time_save_state(state->game_time);
    random_save_state(state->random_iv);
    building_count_save_state(state->building_count_industry,
        state->building_count_culture1,
        state->building_count_culture2,
        state->building_count_culture3,
        state->building_count_military,
        state->building_count_support);

    scenario_emperor_change_save_state(state->emperor_change_time, state->emperor_change_state);
    empire_save_state(state->empire);
    empire_city_save_state(state->empire_cities);
    trade_prices_save_state(state->trade_prices);
    figure_name_save_state(state->figure_names);
    city_culture_save_state(state->culture_coverage);

    scenario_save_state(state->scenario);

    scenario_criteria_save_state(state->max_game_year);
    scenario_earthquake_save_state(state->earthquake);
    city_message_save_state(state->messages, state->message_extra,
        state->message_counts, state->message_delays,
        state->population_messages);
    sound_city_save_state(state->city_sounds);
    traders_save_state(state->figure_traders);

    building_list_save_state(state->building_list_small, state->building_list_large,
        state->building_list_burning, state->building_list_burning_totals);

    tutorial_save_state(state->tutorial_part1, state->tutorial_part2, state->tutorial_part3);

    building_storage_save_state(state->building_storages);
    scenario_gladiator_revolt_save_state(state->gladiator_revolt);
    trade_routes_save_state(state->trade_route_limit, state->trade_route_traded);
    map_routing_save_state(state->routing_counters);
    enemy_armies_save_state(state->enemy_armies, state->enemy_army_totals);
    scenario_invasion_save_state(state->last_invasion_id, state->invasion_warnings);
    map_bookmark_save_state(state->bookmarks);

    buffer_skip(state->end_marker, 284);
}
