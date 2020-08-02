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
#include "scenario/data.h"
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

static const int GRID_U8  = GRID_SIZE * GRID_SIZE * sizeof(uint8_t);
static const int GRID_U16 = GRID_SIZE * GRID_SIZE * sizeof(uint16_t);

static const int SAVE_GAME_VERSION = 0x76;
static const uint16_t SCENARIO_VERSION = 0x01;
static const uint16_t SENTINEL = 0xCAFE;

static char compress_buffer[COMPRESS_BUFFER_SIZE];

static int savegame_version;

typedef struct {
    buffer buf;
    int compressed;
} file_piece;

typedef struct {
    buffer *file_version;
    buffer *graphic_ids;
    buffer *edge;
    buffer *terrain;
    buffer *bitfields;
    buffer *random;
    buffer *elevation;
    buffer *random_iv;
    buffer *camera;
    buffer *scenario;
    buffer *end_marker;
} scenario_state;

typedef struct {
    int num_pieces;
    file_piece pieces[11];
    scenario_state state;
} scenario_data;

typedef struct {
    buffer *scenario_campaign_mission;
    buffer *file_version;
    buffer *image_grid;
    buffer *edge_grid;
    buffer *building_grid;
    buffer *terrain_grid;
    buffer *aqueduct_grid;
    buffer *figure_grid;
    buffer *bitfields_grid;
    buffer *sprite_grid;
    buffer *random_grid;
    buffer *desirability_grid;
    buffer *elevation_grid;
    buffer *building_damage_grid;
    buffer *aqueduct_backup_grid;
    buffer *sprite_backup_grid;
    buffer *figures;
    buffer *route_figures;
    buffer *route_paths;
    buffer *formations;
    buffer *formation_totals;
    buffer *city_data;
    buffer *city_faction_unknown;
    buffer *player_name;
    buffer *city_faction;
    buffer *buildings;
    buffer *city_view_orientation;
    buffer *game_time;
    buffer *building_extra_highest_id_ever;
    buffer *random_iv;
    buffer *city_view_camera;
    buffer *building_count_culture1;
    buffer *city_graph_order;
    buffer *emperor_change_time;
    buffer *empire;
    buffer *empire_cities;
    buffer *building_count_industry;
    buffer *trade_prices;
    buffer *figure_names;
    buffer *culture_coverage;
    buffer *scenario;
    buffer *max_game_year;
    buffer *earthquake;
    buffer *emperor_change_state;
    buffer *messages;
    buffer *message_extra;
    buffer *population_messages;
    buffer *message_counts;
    buffer *message_delays;
    buffer *building_list_burning_totals;
    buffer *figure_sequence;
    buffer *scenario_settings;
    buffer *invasion_warnings;
    buffer *scenario_is_custom;
    buffer *city_sounds;
    buffer *building_extra_highest_id;
    buffer *figure_traders;
    buffer *building_list_burning;
    buffer *building_list_small;
    buffer *building_list_large;
    buffer *tutorial_part1;
    buffer *building_count_military;
    buffer *enemy_army_totals;
    buffer *building_storages;
    buffer *building_count_culture2;
    buffer *building_count_support;
    buffer *tutorial_part2;
    buffer *gladiator_revolt;
    buffer *trade_route_limit;
    buffer *trade_route_traded;
    buffer *building_barracks_tower_sentry;
    buffer *building_extra_sequence;
    buffer *routing_counters;
    buffer *building_count_culture3;
    buffer *enemy_armies;
    buffer *city_entry_exit_xy;
    buffer *last_invasion_id;
    buffer *building_extra_corrupt_houses;
    buffer *scenario_name;
    buffer *bookmarks;
    buffer *tutorial_part3;
    buffer *city_entry_exit_grid_offset;
    buffer *end_marker;
} savegame_state;

static struct {
    int num_pieces;
    file_piece pieces[100];
    savegame_state state;
} savegame_data = {0};

static void init_file_piece(file_piece *piece, int size, int compressed)
{
    piece->compressed = compressed;
    void *data = malloc(size);
    memset(data, 0, size);
    buffer_init(&piece->buf, data, size);
}

static buffer *create_scenario_piece(scenario_data *data, int size)
{
    file_piece *piece = &data->pieces[data->num_pieces++];
    init_file_piece(piece, size, 0);
    return &piece->buf;
}

static buffer *create_savegame_piece(int size, int compressed)
{
    file_piece *piece = &savegame_data.pieces[savegame_data.num_pieces++];
    init_file_piece(piece, size, compressed);
    return &piece->buf;
}

static void init_scenario_data_legacy(scenario_data *data)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }

    scenario_state *state = &data->state;                    // classic map sizes:
    state->graphic_ids = create_scenario_piece(data, 52488); // 162x162 x 2 bytes
    state->edge = create_scenario_piece(data, 26244);        // 162x162 x 1 byte
    state->terrain = create_scenario_piece(data, 52488);
    state->bitfields = create_scenario_piece(data, 26244);
    state->random = create_scenario_piece(data, 26244);
    state->elevation = create_scenario_piece(data, 26244);
    state->random_iv = create_scenario_piece(data, 8);
    state->camera = create_scenario_piece(data, 8);
    state->scenario = create_scenario_piece(data, 1720);
    state->end_marker = create_scenario_piece(data, 4);
}

static void init_scenario_data_current(scenario_data *data)
{
    if (data->num_pieces > 0) {
        for (int i = 0; i < data->num_pieces; i++) {
            buffer_reset(&data->pieces[i].buf);
            free(data->pieces[i].buf.data);
        }
        data->num_pieces = 0;
    }
    scenario_state *state = &data->state;
    state->file_version = create_scenario_piece(data, 4);
    state->graphic_ids = create_scenario_piece(data, GRID_U16);
    state->edge = create_scenario_piece(data, GRID_U8);
    state->terrain = create_scenario_piece(data, GRID_U16);
    state->bitfields = create_scenario_piece(data, GRID_U8);
    state->random = create_scenario_piece(data, GRID_U8);
    state->elevation = create_scenario_piece(data, GRID_U8);
    state->random_iv = create_scenario_piece(data, 8);
    state->camera = create_scenario_piece(data, 8);
    state->scenario = create_scenario_piece(data, 1720);
    state->end_marker = create_scenario_piece(data, 4);
}

static void init_savegame_data(void)
{
    if (savegame_data.num_pieces > 0) {
        for (int i = 0; i < savegame_data.num_pieces; i++) {
            buffer_reset(&savegame_data.pieces[i].buf);
            free(savegame_data.pieces[i].buf.data);
        }
        //return;
        savegame_data.num_pieces = 0;
    }
    savegame_state *state = &savegame_data.state;
    state->scenario_campaign_mission = create_savegame_piece(4, 0);
    state->file_version = create_savegame_piece(4, 0);
    state->image_grid = create_savegame_piece(52488, 1);
    state->edge_grid = create_savegame_piece(26244, 1);
    state->building_grid = create_savegame_piece(52488, 1);
    state->terrain_grid = create_savegame_piece(52488, 1);
    state->aqueduct_grid = create_savegame_piece(26244, 1);
    state->figure_grid = create_savegame_piece(52488, 1);
    state->bitfields_grid = create_savegame_piece(26244, 1);
    state->sprite_grid = create_savegame_piece(26244, 1);
    state->random_grid = create_savegame_piece(26244, 0);
    state->desirability_grid = create_savegame_piece(26244, 1);
    state->elevation_grid = create_savegame_piece(26244, 1);
    state->building_damage_grid = create_savegame_piece(26244, 1);
    state->aqueduct_backup_grid = create_savegame_piece(26244, 1);
    state->sprite_backup_grid = create_savegame_piece(26244, 1);
    state->figures = create_savegame_piece(128000, 1);
    state->route_figures = create_savegame_piece(1200, 1);
    state->route_paths = create_savegame_piece(300000, 1);
    state->formations = create_savegame_piece(6400, 1);
    state->formation_totals = create_savegame_piece(12, 0);
    state->city_data = create_savegame_piece(36136, 1);
    state->city_faction_unknown = create_savegame_piece(2, 0);
    state->player_name = create_savegame_piece(64, 0);
    state->city_faction = create_savegame_piece(4, 0);
    state->buildings = create_savegame_piece(256000, 1);
    state->city_view_orientation = create_savegame_piece(4, 0);
    state->game_time = create_savegame_piece(20, 0);
    state->building_extra_highest_id_ever = create_savegame_piece(8, 0);
    state->random_iv = create_savegame_piece(8, 0);
    state->city_view_camera = create_savegame_piece(8, 0);
    state->building_count_culture1 = create_savegame_piece(132, 0);
    state->city_graph_order = create_savegame_piece(8, 0);
    state->emperor_change_time = create_savegame_piece(8, 0);
    state->empire = create_savegame_piece(12, 0);
    state->empire_cities = create_savegame_piece(2706, 1);
    state->building_count_industry = create_savegame_piece(128, 0);
    state->trade_prices = create_savegame_piece(128, 0);
    state->figure_names = create_savegame_piece(84, 0);
    state->culture_coverage = create_savegame_piece(60, 0);
    state->scenario = create_savegame_piece(1720, 0);
    state->max_game_year = create_savegame_piece(4, 0);
    state->earthquake = create_savegame_piece(60, 0);
    state->emperor_change_state = create_savegame_piece(4, 0);
    state->messages = create_savegame_piece(16000, 1);
    state->message_extra = create_savegame_piece(12, 0);
    state->population_messages = create_savegame_piece(10, 0);
    state->message_counts = create_savegame_piece(80, 0);
    state->message_delays = create_savegame_piece(80, 0);
    state->building_list_burning_totals = create_savegame_piece(8, 0);
    state->figure_sequence = create_savegame_piece(4, 0);
    state->scenario_settings = create_savegame_piece(12, 0);
    state->invasion_warnings = create_savegame_piece(3232, 1);
    state->scenario_is_custom = create_savegame_piece(4, 0);
    state->city_sounds = create_savegame_piece(8960, 0);
    state->building_extra_highest_id = create_savegame_piece(4, 0);
    state->figure_traders = create_savegame_piece(4804, 0);
    state->building_list_burning = create_savegame_piece(1000, 1);
    state->building_list_small = create_savegame_piece(1000, 1);
    state->building_list_large = create_savegame_piece(4000, 1);
    state->tutorial_part1 = create_savegame_piece(32, 0);
    state->building_count_military = create_savegame_piece(16, 0);
    state->enemy_army_totals = create_savegame_piece(20, 0);
    state->building_storages = create_savegame_piece(6400, 0);
    state->building_count_culture2 = create_savegame_piece(32, 0);
    state->building_count_support = create_savegame_piece(24, 0);
    state->tutorial_part2 = create_savegame_piece(4, 0);
    state->gladiator_revolt = create_savegame_piece(16, 0);
    state->trade_route_limit = create_savegame_piece(1280, 1);
    state->trade_route_traded = create_savegame_piece(1280, 1);
    state->building_barracks_tower_sentry = create_savegame_piece(4, 0);
    state->building_extra_sequence = create_savegame_piece(4, 0);
    state->routing_counters = create_savegame_piece(16, 0);
    state->building_count_culture3 = create_savegame_piece(40, 0);
    state->enemy_armies = create_savegame_piece(900, 0);
    state->city_entry_exit_xy = create_savegame_piece(16, 0);
    state->last_invasion_id = create_savegame_piece(2, 0);
    state->building_extra_corrupt_houses = create_savegame_piece(8, 0);
    state->scenario_name = create_savegame_piece(65, 0);
    state->bookmarks = create_savegame_piece(32, 0);
    state->tutorial_part3 = create_savegame_piece(4, 0);
    state->city_entry_exit_grid_offset = create_savegame_piece(8, 0);
    state->end_marker = create_savegame_piece(284, 0); // 71x 4-bytes emptiness
}

static void init_savegame_data_expanded(void)
{
    if (savegame_data.num_pieces > 0) {
        for (int i = 0; i < savegame_data.num_pieces; i++) {
            buffer_reset(&savegame_data.pieces[i].buf);
            free(savegame_data.pieces[i].buf.data);
        }
        //return;
        savegame_data.num_pieces = 0;
    }
    savegame_state *state = &savegame_data.state;
    state->scenario_campaign_mission = create_savegame_piece(4, 0);
    state->file_version = create_savegame_piece(4, 0);
    state->image_grid = create_savegame_piece(GRID_U16, 1);
    state->edge_grid = create_savegame_piece(GRID_U8, 1);
    state->building_grid = create_savegame_piece(GRID_U16, 1);
    state->terrain_grid = create_savegame_piece(GRID_U16, 1);
    state->aqueduct_grid = create_savegame_piece(GRID_U8, 1);
    state->figure_grid = create_savegame_piece(GRID_U16, 1);
    state->bitfields_grid = create_savegame_piece(GRID_U8, 1);
    state->sprite_grid = create_savegame_piece(GRID_U8, 1);
    state->random_grid = create_savegame_piece(GRID_U8, 0);
    state->desirability_grid = create_savegame_piece(GRID_U8, 1);
    state->elevation_grid = create_savegame_piece(GRID_U8, 1);
    state->building_damage_grid = create_savegame_piece(GRID_U8, 1);
    state->aqueduct_backup_grid = create_savegame_piece(GRID_U8, 1);
    state->sprite_backup_grid = create_savegame_piece(GRID_U8, 1);
    state->figures = create_savegame_piece(640000, 1);
    state->route_figures = create_savegame_piece(6000, 1);
    state->route_paths = create_savegame_piece(1500000, 1);
    state->formations = create_savegame_piece(32000, 1);
    state->formation_totals = create_savegame_piece(12, 0);
    state->city_data = create_savegame_piece(36136, 1);
    state->city_faction_unknown = create_savegame_piece(2, 0);
    state->player_name = create_savegame_piece(64, 0);
    state->city_faction = create_savegame_piece(4, 0);
    state->buildings = create_savegame_piece(1280000, 1);
    state->city_view_orientation = create_savegame_piece(4, 0);
    state->game_time = create_savegame_piece(20, 0);
    state->building_extra_highest_id_ever = create_savegame_piece(8, 0);
    state->random_iv = create_savegame_piece(8, 0);
    state->city_view_camera = create_savegame_piece(8, 0);
    state->building_count_culture1 = create_savegame_piece(132, 0);
    state->city_graph_order = create_savegame_piece(8, 0);
    state->emperor_change_time = create_savegame_piece(8, 0);
    state->empire = create_savegame_piece(12, 0);
    state->empire_cities = create_savegame_piece(2706, 1);
    state->building_count_industry = create_savegame_piece(128, 0);
    state->trade_prices = create_savegame_piece(128, 0);
    state->figure_names = create_savegame_piece(84, 0);
    state->culture_coverage = create_savegame_piece(60, 0);
    state->scenario = create_savegame_piece(1720, 0);
    state->max_game_year = create_savegame_piece(4, 0);
    state->earthquake = create_savegame_piece(60, 0);
    state->emperor_change_state = create_savegame_piece(4, 0);
    state->messages = create_savegame_piece(16000, 1);
    state->message_extra = create_savegame_piece(12, 0);
    state->population_messages = create_savegame_piece(10, 0);
    state->message_counts = create_savegame_piece(80, 0);
    state->message_delays = create_savegame_piece(80, 0);
    state->building_list_burning_totals = create_savegame_piece(8, 0);
    state->figure_sequence = create_savegame_piece(4, 0);
    state->scenario_settings = create_savegame_piece(12, 0);
    state->invasion_warnings = create_savegame_piece(3232, 1);
    state->scenario_is_custom = create_savegame_piece(4, 0);
    state->city_sounds = create_savegame_piece(8960, 0);
    state->building_extra_highest_id = create_savegame_piece(4, 0);
    state->figure_traders = create_savegame_piece(4804, 0);
    state->building_list_burning = create_savegame_piece(5000, 1);
    state->building_list_small = create_savegame_piece(5000, 1);
    state->building_list_large = create_savegame_piece(20000, 1);
    state->tutorial_part1 = create_savegame_piece(32, 0);
    state->building_count_military = create_savegame_piece(16, 0);
    state->enemy_army_totals = create_savegame_piece(20, 0);
    state->building_storages = create_savegame_piece(32000, 0);
    state->building_count_culture2 = create_savegame_piece(32, 0);
    state->building_count_support = create_savegame_piece(24, 0);
    state->tutorial_part2 = create_savegame_piece(4, 0);
    state->gladiator_revolt = create_savegame_piece(16, 0);
    state->trade_route_limit = create_savegame_piece(1280, 1);
    state->trade_route_traded = create_savegame_piece(1280, 1);
    state->building_barracks_tower_sentry = create_savegame_piece(4, 0);
    state->building_extra_sequence = create_savegame_piece(4, 0);
    state->routing_counters = create_savegame_piece(16, 0);
    state->building_count_culture3 = create_savegame_piece(40, 0);
    state->enemy_armies = create_savegame_piece(900, 0);
    state->city_entry_exit_xy = create_savegame_piece(16, 0);
    state->last_invasion_id = create_savegame_piece(2, 0);
    state->building_extra_corrupt_houses = create_savegame_piece(8, 0);
    state->scenario_name = create_savegame_piece(65, 0);
    state->bookmarks = create_savegame_piece(32, 0);
    state->tutorial_part3 = create_savegame_piece(4, 0);
    state->city_entry_exit_grid_offset = create_savegame_piece(8, 0);
    state->end_marker = create_savegame_piece(284, 0); // 71x 4-bytes emptiness
}

static void scenario_version_save_state(buffer *buf)
{
    buffer_write_u16(buf, SENTINEL);
    buffer_write_u16(buf, SCENARIO_VERSION);
}

static void scenario_version_load_state(buffer *buf)
{
    buffer_skip(buf, 4);
}

static void scenario_load_from_state(scenario_state *file, int version)
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

static void scenario_save_to_state(scenario_state *file)
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

static void savegame_load_from_state(savegame_state *state)
{
    savegame_version = buffer_read_i32(state->file_version);

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

static void savegame_save_to_state(savegame_state *state)
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

static int fetch_scenario_version(FILE *fp)
{
    uint16_t sentinel_check = 0;
    fread(&sentinel_check, 2, 1, fp);
    if (sentinel_check != SENTINEL) {
        fseek(fp, 0, SEEK_SET);
        return 0; // shouldn't be here for mapx files
    }

    uint16_t version = 0;
    fread(&version, 2, 1, fp);
    fseek(fp, 0, SEEK_SET);

    return version;
}

static void scenario_migrate_mapsize(scenario_data *new_data, scenario_data *old_data, int old_size, int new_size, short has_ver)
{
    // migrating map sizes requires replacing scenario pieces:
    // 1 graphic_ids (16bit)
    // 2 edge (8bit)
    // 3 terrain (16bit)
    // 4 bitfields (8bit)
    // 5 random (8bit)
    // 6 elevation (8bit)
    // if has_ver is 0 then data won't have the version buffer, read from piece[0-5] instead of [1-6]
    // scenario map information will also have to be adjusted (start_offset, border_size)

    int piece_bytesize[] = {
        2,1,2,1,1,1
    };

    // there will be some throw-away work here to load needed buffers/map information
    init_scenario_data_current(new_data);
    
    // find new top left starting index of larger, centered map
    int new_start_idx = (new_size - old_size) / 2 * new_size + (new_size - old_size) / 2;

    for (int i = 0; i < old_data->num_pieces; ++i) {
        buffer_reset(&new_data->pieces[i + 1].buf);
        if (i < 6) {
            int bytesize = piece_bytesize[i];
            buffer *old_buf = &old_data->pieces[i + has_ver].buf;
            buffer *new_buf = &new_data->pieces[i + 1].buf;

            for (int y = 0; y < old_size; ++y) {
                int old_idx = y * old_size * bytesize;
                int new_idx = (new_start_idx * bytesize) + (y * new_size * bytesize);

                buffer_set(new_buf, new_idx);
                buffer_write_raw(new_buf, &old_buf->data[old_idx], old_size * bytesize);
            }
        } else {
            buffer_write_raw(&new_data->pieces[i + 1].buf, old_data->pieces[i + has_ver].buf.data, old_data->pieces[i + has_ver].buf.size);
        }
        buffer_reset(&new_data->pieces[i + 1].buf);
    }

    // store resized information in original state
    // fetch incoming map data from old_data
    int width, height;
    scenario_load_state(old_data->state.scenario);
    scenario_map_init();
    map_grid_size(&width, &height);

    // store modified map data back into new_data
    scenario.map.grid_border_size = new_size - scenario.map.width;
    scenario.map.grid_start = (new_size - scenario.map.height) / 2 * new_size + (new_size - scenario.map.width) / 2;
    scenario_save_state(new_data->state.scenario);

    for (int i = 0; i < new_data->num_pieces; ++i) {
        buffer_reset(&new_data->pieces[i].buf);
    }
}

static int scenario_migrate_and_load_from_state(scenario_data *data, int version)
{
    scenario_data migrated_data = {0};
    switch (version) {
        case 0: // classic maps are 162x162, have no version buffer
            log_info("Migrating legacy map", 0, 0);
            scenario_migrate_mapsize(&migrated_data, data, 162, GRID_SIZE, 0);
            break;
    }

    scenario_load_from_state(&migrated_data.state, version);
    return 0;
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
    
    if (file_has_extension(filename, "mapx")) {
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
        // migrate buffers before loading state
        if (!scenario_migrate_and_load_from_state(&data, scenario_ver)) {
            log_error("Failed to migrate and load scenario current version", 0, 0);
            return 0;
        } else {
            return 1;
        }
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

static int savegame_read_from_file(FILE *fp)
{
    for (int i = 0; i < savegame_data.num_pieces; i++) {
        file_piece *piece = &savegame_data.pieces[i];
        int result = 0;
        if (piece->compressed) {
            result = read_compressed_chunk(fp, piece->buf.data, piece->buf.size);
        } else {
            result = fread(piece->buf.data, 1, piece->buf.size, fp) == piece->buf.size;
        }
        // The last piece may be smaller than buf.size
        if (!result && i != (savegame_data.num_pieces - 1)) {
            log_info("Incorrect buffer size, got.", 0, result);
            log_info("Incorrect buffer size, expected." , 0,piece->buf.size);
            return 0;
        }
    }
    return 1;
}

static void savegame_write_to_file(FILE *fp)
{
    for (int i = 0; i < savegame_data.num_pieces; i++) {
        file_piece *piece = &savegame_data.pieces[i];
        if (piece->compressed) {
            write_compressed_chunk(fp, piece->buf.data, piece->buf.size);
        } else {
            fwrite(piece->buf.data, 1, piece->buf.size, fp);
        }
    }
}
int game_file_io_read_saved_game(const char *filename, int offset)
{
    if (file_has_extension(filename,"svx")) {
        init_savegame_data_expanded();
        log_info("Loading saved game new format.", filename, 0);

    } else {
        log_info("Loading saved game old format.", filename, 0);
        init_savegame_data();
    }

    log_info("Loading saved game", filename, 0);
    FILE *fp = file_open(dir_get_file(filename, NOT_LOCALIZED), "rb");
    if (!fp) {
        log_error("Unable to load game, unable to open file.", 0, 0);
        return 0;
    }
    if (offset) {
        fseek(fp, offset, SEEK_SET);
    }
    int result = savegame_read_from_file(fp);
    file_close(fp);
    if (!result) {
        log_error("Unable to load game, unable to read savefile.", 0, 0);
        return 0;
    }
    savegame_load_from_state(&savegame_data.state);
    return 1;
}

int game_file_io_write_saved_game(const char *filename)
{
    init_savegame_data_expanded();

    log_info("Saving game", filename, 0);
    savegame_version = SAVE_GAME_VERSION;
    savegame_save_to_state(&savegame_data.state);

    FILE *fp = file_open(filename, "wb");
    if (!fp) {
        log_error("Unable to save game", 0, 0);
        return 0;
    }
    savegame_write_to_file(fp);
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
