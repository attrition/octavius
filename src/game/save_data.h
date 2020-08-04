#ifndef GAME_SAVE_DATA_H
#define GAME_SAVE_DATA_H

#include "core/buffer.h"

#define SAVE_GAME_VERSION_LEGACY 0x66
#define SAVE_GAME_VERSION_AUG_V1 0x76
#define SAVE_GAME_VERSION 0x77
#define SCENARIO_VERSION_LEGACY 0x00
#define SCENARIO_VERSION 0x01

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

typedef struct {
    int num_pieces;
    file_piece pieces[100];
    savegame_state state;
} savegame_data;

void save_data_scenario_init_data(scenario_data *data, int version);
void save_data_scenario_load_from_state(scenario_state *file, int version);
void save_data_scenario_save_to_state(scenario_state *file);

void save_data_savegame_init_data_legacy(savegame_data *data);
void save_data_savegame_init_data_augustus(savegame_data *data, int version);
void save_data_savegame_load_from_state(savegame_state *state);
void save_data_savegame_save_to_state(savegame_state *state, int savegame_version);

#endif
