#include "file_io.h"

#include "building/barracks.h"
#include "building/count.h"
#include "building/list.h"
#include "building/storage.h"
#include "city/culture.h"
#include "city/data.h"
#include "core/file.h"
#include "core/log.h"
#include "city/message.h"
#include "city/view.h"
#include "core/dir.h"
#include "core/random.h"
#include "core/zip.h"
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

#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define COMPRESS_BUFFER_SIZE 3000000
#define UNCOMPRESSED 0x80000000

static char compress_buffer[COMPRESS_BUFFER_SIZE];

void init_scenario_data_legacy(scenario_data *data)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }

    scenario_state *state = &data->state;                              // classic map sizes:
    state->graphic_ids = save_data_create_scenario_piece(data, 52488); // 162x162 x 2 bytes
    state->edge = save_data_create_scenario_piece(data, 26244);        // 162x162 x 1 byte
    state->terrain = save_data_create_scenario_piece(data, 52488);
    state->bitfields = save_data_create_scenario_piece(data, 26244);
    state->random = save_data_create_scenario_piece(data, 26244);
    state->elevation = save_data_create_scenario_piece(data, 26244);
    state->random_iv = save_data_create_scenario_piece(data, 8);
    state->camera = save_data_create_scenario_piece(data, 8);
    state->scenario = save_data_create_scenario_piece(data, 1720);
    state->end_marker = save_data_create_scenario_piece(data, 4);
}

void init_scenario_data_current(scenario_data *data)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }

    int grid_u8 = GRID_SIZE * GRID_SIZE * sizeof(uint8_t);

    scenario_state *state = &data->state;
    state->file_version = save_data_create_scenario_piece(data, 4);
    state->graphic_ids = save_data_create_scenario_piece(data, grid_u8 * 2);
    state->edge = save_data_create_scenario_piece(data, grid_u8);
    state->terrain = save_data_create_scenario_piece(data, grid_u8 * 2);
    state->bitfields = save_data_create_scenario_piece(data, grid_u8);
    state->random = save_data_create_scenario_piece(data, grid_u8);
    state->elevation = save_data_create_scenario_piece(data, grid_u8);
    state->random_iv = save_data_create_scenario_piece(data, 8);
    state->camera = save_data_create_scenario_piece(data, 8);
    state->scenario = save_data_create_scenario_piece(data, 1720);
    state->end_marker = save_data_create_scenario_piece(data, 4);
}

void init_savegame_data(savegame_data *data)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }
    savegame_state *state = &data->state;
    state->scenario_campaign_mission = save_data_create_savegame_piece(data, 4, 0);
    state->file_version = save_data_create_savegame_piece(data, 4, 0);
    state->image_grid = save_data_create_savegame_piece(data, 52488, 1);
    state->edge_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->building_grid = save_data_create_savegame_piece(data, 52488, 1);
    state->terrain_grid = save_data_create_savegame_piece(data, 52488, 1);
    state->aqueduct_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->figure_grid = save_data_create_savegame_piece(data, 52488, 1);
    state->bitfields_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->sprite_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->random_grid = save_data_create_savegame_piece(data, 26244, 0);
    state->desirability_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->elevation_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->building_damage_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->aqueduct_backup_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->sprite_backup_grid = save_data_create_savegame_piece(data, 26244, 1);
    state->figures = save_data_create_savegame_piece(data, 128000, 1);
    state->route_figures = save_data_create_savegame_piece(data, 1200, 1);
    state->route_paths = save_data_create_savegame_piece(data, 300000, 1);
    state->formations = save_data_create_savegame_piece(data, 6400, 1);
    state->formation_totals = save_data_create_savegame_piece(data, 12, 0);
    state->city_data = save_data_create_savegame_piece(data, 36136, 1);
    state->city_faction_unknown = save_data_create_savegame_piece(data, 2, 0);
    state->player_name = save_data_create_savegame_piece(data, 64, 0);
    state->city_faction = save_data_create_savegame_piece(data, 4, 0);
    state->buildings = save_data_create_savegame_piece(data, 256000, 1);
    state->city_view_orientation = save_data_create_savegame_piece(data, 4, 0);
    state->game_time = save_data_create_savegame_piece(data, 20, 0);
    state->building_extra_highest_id_ever = save_data_create_savegame_piece(data, 8, 0);
    state->random_iv = save_data_create_savegame_piece(data, 8, 0);
    state->city_view_camera = save_data_create_savegame_piece(data, 8, 0);
    state->building_count_culture1 = save_data_create_savegame_piece(data, 132, 0);
    state->city_graph_order = save_data_create_savegame_piece(data, 8, 0);
    state->emperor_change_time = save_data_create_savegame_piece(data, 8, 0);
    state->empire = save_data_create_savegame_piece(data, 12, 0);
    state->empire_cities = save_data_create_savegame_piece(data, 2706, 1);
    state->building_count_industry = save_data_create_savegame_piece(data, 128, 0);
    state->trade_prices = save_data_create_savegame_piece(data, 128, 0);
    state->figure_names = save_data_create_savegame_piece(data, 84, 0);
    state->culture_coverage = save_data_create_savegame_piece(data, 60, 0);
    state->scenario = save_data_create_savegame_piece(data, 1720, 0);
    state->max_game_year = save_data_create_savegame_piece(data, 4, 0);
    state->earthquake = save_data_create_savegame_piece(data, 60, 0);
    state->emperor_change_state = save_data_create_savegame_piece(data, 4, 0);
    state->messages = save_data_create_savegame_piece(data, 16000, 1);
    state->message_extra = save_data_create_savegame_piece(data, 12, 0);
    state->population_messages = save_data_create_savegame_piece(data, 10, 0);
    state->message_counts = save_data_create_savegame_piece(data, 80, 0);
    state->message_delays = save_data_create_savegame_piece(data, 80, 0);
    state->building_list_burning_totals = save_data_create_savegame_piece(data, 8, 0);
    state->figure_sequence = save_data_create_savegame_piece(data, 4, 0);
    state->scenario_settings = save_data_create_savegame_piece(data, 12, 0);
    state->invasion_warnings = save_data_create_savegame_piece(data, 3232, 1);
    state->scenario_is_custom = save_data_create_savegame_piece(data, 4, 0);
    state->city_sounds = save_data_create_savegame_piece(data, 8960, 0);
    state->building_extra_highest_id = save_data_create_savegame_piece(data, 4, 0);
    state->figure_traders = save_data_create_savegame_piece(data, 4804, 0);
    state->building_list_burning = save_data_create_savegame_piece(data, 1000, 1);
    state->building_list_small = save_data_create_savegame_piece(data, 1000, 1);
    state->building_list_large = save_data_create_savegame_piece(data, 4000, 1);
    state->tutorial_part1 = save_data_create_savegame_piece(data, 32, 0);
    state->building_count_military = save_data_create_savegame_piece(data, 16, 0);
    state->enemy_army_totals = save_data_create_savegame_piece(data, 20, 0);
    state->building_storages = save_data_create_savegame_piece(data, 6400, 0);
    state->building_count_culture2 = save_data_create_savegame_piece(data, 32, 0);
    state->building_count_support = save_data_create_savegame_piece(data, 24, 0);
    state->tutorial_part2 = save_data_create_savegame_piece(data, 4, 0);
    state->gladiator_revolt = save_data_create_savegame_piece(data, 16, 0);
    state->trade_route_limit = save_data_create_savegame_piece(data, 1280, 1);
    state->trade_route_traded = save_data_create_savegame_piece(data, 1280, 1);
    state->building_barracks_tower_sentry = save_data_create_savegame_piece(data, 4, 0);
    state->building_extra_sequence = save_data_create_savegame_piece(data, 4, 0);
    state->routing_counters = save_data_create_savegame_piece(data, 16, 0);
    state->building_count_culture3 = save_data_create_savegame_piece(data, 40, 0);
    state->enemy_armies = save_data_create_savegame_piece(data, 900, 0);
    state->city_entry_exit_xy = save_data_create_savegame_piece(data, 16, 0);
    state->last_invasion_id = save_data_create_savegame_piece(data, 2, 0);
    state->building_extra_corrupt_houses = save_data_create_savegame_piece(data, 8, 0);
    state->scenario_name = save_data_create_savegame_piece(data, 65, 0);
    state->bookmarks = save_data_create_savegame_piece(data, 32, 0);
    state->tutorial_part3 = save_data_create_savegame_piece(data, 4, 0);
    state->city_entry_exit_grid_offset = save_data_create_savegame_piece(data, 8, 0);
    state->end_marker = save_data_create_savegame_piece(data, 284, 0); // 71x 4-bytes emptiness
}

void init_savegame_data_augustus(savegame_data *data, int version)
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
    state->scenario_campaign_mission = save_data_create_savegame_piece(data, 4, 0);
    state->file_version = save_data_create_savegame_piece(data, 4, 0);
    state->image_grid = save_data_create_savegame_piece(data, grid_u8 * 2, 1);
    state->edge_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->building_grid = save_data_create_savegame_piece(data, grid_u8 * 2, 1);
    state->terrain_grid = save_data_create_savegame_piece(data, grid_u8 * 2, 1);
    state->aqueduct_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->figure_grid = save_data_create_savegame_piece(data, grid_u8 * 2, 1);
    state->bitfields_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->sprite_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->random_grid = save_data_create_savegame_piece(data, grid_u8, 0);
    state->desirability_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->elevation_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->building_damage_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->aqueduct_backup_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->sprite_backup_grid = save_data_create_savegame_piece(data, grid_u8, 1);
    state->figures = save_data_create_savegame_piece(data, 640000, 1);
    state->route_figures = save_data_create_savegame_piece(data, 6000, 1);
    state->route_paths = save_data_create_savegame_piece(data, 1500000, 1);
    state->formations = save_data_create_savegame_piece(data, 32000, 1);
    state->formation_totals = save_data_create_savegame_piece(data, 12, 0);
    state->city_data = save_data_create_savegame_piece(data, 36136, 1);
    state->city_faction_unknown = save_data_create_savegame_piece(data, 2, 0);
    state->player_name = save_data_create_savegame_piece(data, 64, 0);
    state->city_faction = save_data_create_savegame_piece(data, 4, 0);
    state->buildings = save_data_create_savegame_piece(data, 1280000, 1);
    state->city_view_orientation = save_data_create_savegame_piece(data, 4, 0);
    state->game_time = save_data_create_savegame_piece(data, 20, 0);
    state->building_extra_highest_id_ever = save_data_create_savegame_piece(data, 8, 0);
    state->random_iv = save_data_create_savegame_piece(data, 8, 0);
    state->city_view_camera = save_data_create_savegame_piece(data, 8, 0);
    state->building_count_culture1 = save_data_create_savegame_piece(data, 132, 0);
    state->city_graph_order = save_data_create_savegame_piece(data, 8, 0);
    state->emperor_change_time = save_data_create_savegame_piece(data, 8, 0);
    state->empire = save_data_create_savegame_piece(data, 12, 0);
    state->empire_cities = save_data_create_savegame_piece(data, 2706, 1);
    state->building_count_industry = save_data_create_savegame_piece(data, 128, 0);
    state->trade_prices = save_data_create_savegame_piece(data, 128, 0);
    state->figure_names = save_data_create_savegame_piece(data, 84, 0);
    state->culture_coverage = save_data_create_savegame_piece(data, 60, 0);
    state->scenario = save_data_create_savegame_piece(data, 1720, 0);
    state->max_game_year = save_data_create_savegame_piece(data, 4, 0);
    state->earthquake = save_data_create_savegame_piece(data, 60, 0);
    state->emperor_change_state = save_data_create_savegame_piece(data, 4, 0);
    state->messages = save_data_create_savegame_piece(data, 16000, 1);
    state->message_extra = save_data_create_savegame_piece(data, 12, 0);
    state->population_messages = save_data_create_savegame_piece(data, 10, 0);
    state->message_counts = save_data_create_savegame_piece(data, 80, 0);
    state->message_delays = save_data_create_savegame_piece(data, 80, 0);
    state->building_list_burning_totals = save_data_create_savegame_piece(data, 8, 0);
    state->figure_sequence = save_data_create_savegame_piece(data, 4, 0);
    state->scenario_settings = save_data_create_savegame_piece(data, 12, 0);
    state->invasion_warnings = save_data_create_savegame_piece(data, 3232, 1);
    state->scenario_is_custom = save_data_create_savegame_piece(data, 4, 0);
    state->city_sounds = save_data_create_savegame_piece(data, 8960, 0);
    state->building_extra_highest_id = save_data_create_savegame_piece(data, 4, 0);
    state->figure_traders = save_data_create_savegame_piece(data, 4804, 0);
    state->building_list_burning = save_data_create_savegame_piece(data, 5000, 1);
    state->building_list_small = save_data_create_savegame_piece(data, 5000, 1);
    state->building_list_large = save_data_create_savegame_piece(data, 20000, 1);
    state->tutorial_part1 = save_data_create_savegame_piece(data, 32, 0);
    state->building_count_military = save_data_create_savegame_piece(data, 16, 0);
    state->enemy_army_totals = save_data_create_savegame_piece(data, 20, 0);
    state->building_storages = save_data_create_savegame_piece(data, 32000, 0);
    state->building_count_culture2 = save_data_create_savegame_piece(data, 32, 0);
    state->building_count_support = save_data_create_savegame_piece(data, 24, 0);
    state->tutorial_part2 = save_data_create_savegame_piece(data, 4, 0);
    state->gladiator_revolt = save_data_create_savegame_piece(data, 16, 0);
    state->trade_route_limit = save_data_create_savegame_piece(data, 1280, 1);
    state->trade_route_traded = save_data_create_savegame_piece(data, 1280, 1);
    state->building_barracks_tower_sentry = save_data_create_savegame_piece(data, 4, 0);
    state->building_extra_sequence = save_data_create_savegame_piece(data, 4, 0);
    state->routing_counters = save_data_create_savegame_piece(data, 16, 0);
    state->building_count_culture3 = save_data_create_savegame_piece(data, 40, 0);
    state->enemy_armies = save_data_create_savegame_piece(data, 900, 0);
    state->city_entry_exit_xy = save_data_create_savegame_piece(data, 16, 0);
    state->last_invasion_id = save_data_create_savegame_piece(data, 2, 0);
    state->building_extra_corrupt_houses = save_data_create_savegame_piece(data, 8, 0);
    state->scenario_name = save_data_create_savegame_piece(data, 65, 0);
    state->bookmarks = save_data_create_savegame_piece(data, 32, 0);
    state->tutorial_part3 = save_data_create_savegame_piece(data, 4, 0);
    state->city_entry_exit_grid_offset = save_data_create_savegame_piece(data, 8, 0);
    state->end_marker = save_data_create_savegame_piece(data, 284, 0); // 71x 4-bytes emptiness
}

void scenario_version_save_state(buffer *buf)
{
    buffer_write_u16(buf, SCENARIO_VERSION);
}

void scenario_version_load_state(buffer *buf)
{
    buffer_skip(buf, 2);
}

void scenario_load_from_state(scenario_state *file, int version)
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

void scenario_save_to_state(scenario_state *file)
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

void savegame_load_from_state(savegame_state *state)
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

void savegame_save_to_state(savegame_state *state, int savegame_version)
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

int fetch_scenario_version(FILE *fp)
{
    uint16_t version = 0;
    fread(&version, 1, 2, fp);
    fseek(fp, 0, SEEK_SET);

    return version;
}

int game_file_io_read_scenario(const char *filename)
{
    scenario_data data = {0};

    log_info("Loading scenario file", filename, 0);
    FILE *fp = file_open(dir_get_file(filename, NOT_LOCALIZED), "rb");
    if (!fp) {
        return 0;
    }

    log_info("Checking scenario compatibility", 0, 0);
    int scenario_ver = 0;
    
    if (file_has_extension(filename, "mpx")) {
        scenario_ver = fetch_scenario_version(fp);
    }
        
    switch (scenario_ver) {
        case 0:
            log_info("Loading legacy scenario", 0, 0);
            init_scenario_data_legacy(&data);
            break;
        case 1:
            log_info("Loading Augustus scenario, version", 0, scenario_ver);
            init_scenario_data_current(&data);
            break;
        default:
            return 0; // unhandled version, don't attempt to load
    }

    for (int i = 0; i < data.num_pieces; i++) {
        if (fread(data.pieces[i].buf.data, 1, data.pieces[i].buf.size, fp) != data.pieces[i].buf.size) {
            log_error("Unable to load scenario", filename, 0);
            file_close(fp);
            return 0;
        }
    }
    file_close(fp);

    if (scenario_ver != SCENARIO_VERSION) {
        // migrate buffers and load state
        scenario_data migrated_data = {0};
        init_scenario_data_current(&migrated_data);
        if (!migrate_scenario_and_load_from_state(&migrated_data, &data, scenario_ver)) {
            log_error("Failed to migrate and load scenario current version", 0, 0);
            return 0;
        }
        scenario_load_from_state(&migrated_data.state, scenario_ver);
        return 1;
    }

    scenario_load_from_state(&data.state, scenario_ver);
    return 1;
}

int game_file_io_write_scenario(const char *filename)
{
    scenario_data data = {0};

    log_info("Saving scenario", filename, 0);

    init_scenario_data_current(&data);
    scenario_save_to_state(&data.state);

    FILE *fp = file_open(filename, "wb");
    if (!fp) {
        log_error("Unable to save scenario", 0, 0);
        return 0;
    }
    for (int i = 0; i < data.num_pieces; i++) {
        fwrite(data.pieces[i].buf.data, 1, data.pieces[i].buf.size, fp);
    }
    file_close(fp);
    return 1;
}

static int read_int32(FILE *fp)
{
    uint8_t data[4];
    if (fread(&data, 1, 4, fp) != 4) {
        return 0;
    }
    buffer buf;
    buffer_init(&buf, data, 4);
    return buffer_read_i32(&buf);
}

static void write_int32(FILE *fp, int value)
{
    uint8_t data[4];
    buffer buf;
    buffer_init(&buf, data, 4);
    buffer_write_i32(&buf, value);
    fwrite(&data, 1, 4, fp);
}

static int read_compressed_chunk(FILE *fp, void *buffer, int bytes_to_read)
{
    if (bytes_to_read > COMPRESS_BUFFER_SIZE) {
        return 0;
    }
    int input_size = read_int32(fp);
    if ((unsigned int) input_size == UNCOMPRESSED) {
        if (fread(buffer, 1, bytes_to_read, fp) != bytes_to_read) {
            return 0;
        }
    } else {
        if (fread(compress_buffer, 1, input_size, fp) != input_size || !zip_decompress(compress_buffer, input_size, buffer, &bytes_to_read)) {
            return 0;
        }
    }
    return 1;
}

static int write_compressed_chunk(FILE *fp, const void *buffer, int bytes_to_write)
{
    if (bytes_to_write > COMPRESS_BUFFER_SIZE) {
        return 0;
    }
    int output_size = COMPRESS_BUFFER_SIZE;
    if (zip_compress(buffer, bytes_to_write, compress_buffer, &output_size)) {
        write_int32(fp, output_size);
        fwrite(compress_buffer, 1, output_size, fp);
    } else {
        // unable to compress: write uncompressed
        write_int32(fp, UNCOMPRESSED);
        fwrite(buffer, 1, bytes_to_write, fp);
    }
    return 1;
}

int savegame_read_from_file(savegame_data *data, FILE *fp)
{
    for (int i = 0; i < data->num_pieces; i++) {
        file_piece *piece = &data->pieces[i];
        int result = 0;
        if (piece->compressed) {
            result = read_compressed_chunk(fp, piece->buf.data, piece->buf.size);
        } else {
            result = fread(piece->buf.data, 1, piece->buf.size, fp) == piece->buf.size;
        }
        // The last piece may be smaller than buf.size
        if (!result && i != (data->num_pieces - 1)) {
            log_info("Incorrect buffer size, got.", 0, result);
            log_info("Incorrect buffer size, expected." , 0,piece->buf.size);
            return 0;
        }
    }
    return 1;
}

void savegame_write_to_file(savegame_data *data, FILE *fp)
{
    for (int i = 0; i < data->num_pieces; i++) {
        file_piece *piece = &data->pieces[i];
        if (piece->compressed) {
            write_compressed_chunk(fp, piece->buf.data, piece->buf.size);
        } else {
            fwrite(piece->buf.data, 1, piece->buf.size, fp);
        }
    }
}

int game_file_io_read_saved_game(const char *filename, int offset)
{
    savegame_data data = {0};

    log_info("Opening saved game file", filename, 0);
    FILE *fp = file_open(dir_get_file(filename, NOT_LOCALIZED), "rb");
    if (!fp) {
        log_error("Unable to load game, unable to open file.", 0, 0);
        return 0;
    }

    // check savegame version
    log_info("Checking saved game compatibility", 0, 0);
    int savegame_version = 0;
    fseek(fp, offset + 4, SEEK_SET);
    fread(&savegame_version, 1, 4, fp);
    log_info("Found savegame version", 0, savegame_version);

    // return to regular savegame processing
    fseek(fp, offset, SEEK_SET);

    switch (savegame_version) {
        case SAVE_GAME_VERSION_CLASSIC:
            log_info("Loading saved game with legacy format.", 0, 0);
            init_savegame_data(&data);
            break;
        case SAVE_GAME_VERSION_AUG_V1:
        case SAVE_GAME_VERSION:
            log_info("Loading saved game with augustus current format.", 0, 0);
            init_savegame_data_augustus(&data, savegame_version);
            break;
    }

    int result = savegame_read_from_file(&data, fp);
    file_close(fp);
    if (!result) {
        log_error("Unable to load game, unable to read savefile.", 0, 0);
        return 0;
    }

    if (savegame_version != SAVE_GAME_VERSION) {
        // migrate buffers and load state
        savegame_data migrated_data = {0};
        init_savegame_data_augustus(&migrated_data, SAVE_GAME_VERSION);
        if (!migrate_savegame_and_load_from_state(&migrated_data, &data, savegame_version)) {
            log_error("Failed to migrate and load savegame current version", 0, 0);
            return 0;
        }
        savegame_load_from_state(&migrated_data.state);
        return 1;
    }

    savegame_load_from_state(&data.state);
    return 1;
}

int game_file_io_write_saved_game(const char *filename)
{
    savegame_data data = {0};
    log_info("Saving game", filename, 0);
    int savegame_version = SAVE_GAME_VERSION;

    init_savegame_data_augustus(&data, savegame_version);
    savegame_save_to_state(&data.state, savegame_version);

    FILE *fp = file_open(filename, "wb");
    if (!fp) {
        log_error("Unable to save game", 0, 0);
        return 0;
    }
    savegame_write_to_file(&data, fp);
    file_close(fp);
    return 1;
}

int game_file_io_delete_saved_game(const char *filename)
{
    log_info("Deleting game", filename, 0);
    int result = file_remove(filename);
    if (!result) {
        log_error("Unable to delete game", 0, 0);
    }
    return result;
}
