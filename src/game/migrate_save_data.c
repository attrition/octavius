#include "migrate_save_data.h"

#include "core/buffer.h"
#include "core/log.h"
#include "game/save_data.h"
#include "map/grid.h"
#include "scenario/data.h"
#include "scenario/map.h"
#include "scenario/scenario.h"

void migrate_buffer_mapsize(buffer *new_buf, buffer *old_buf, int bytesize, int old_size, int new_size)
{
    // find new top left starting index of larger, centered map
    int new_start_idx = (new_size - old_size) / 2 * new_size + (new_size - old_size) / 2;
    for (int y = 0; y < old_size; ++y) {
        int old_idx = y * old_size * bytesize;
        int new_idx = (new_start_idx * bytesize) + (y * new_size * bytesize);

        buffer_set(new_buf, new_idx);
        buffer_write_raw(new_buf, &old_buf->data[old_idx], old_size * bytesize);
    }
}

void migrate_scenario_map_data(buffer *new_scenario, buffer *old_scenario, int old_size, int new_size)
{
    // store resized information in original state
    // fetch incoming map data from old_data
    int width, height;
    scenario_load_state(old_scenario);
    scenario_map_init();
    map_grid_size(&width, &height);

    // store modified map data back into new_data
    scenario.map.grid_border_size = new_size - scenario.map.width;
    scenario.map.grid_start = (new_size - scenario.map.height) / 2 * new_size + (new_size - scenario.map.width) / 2;
    scenario_save_state(new_scenario);
}

void migrate_scenario_mapsize(scenario_data *new_data, scenario_data *old_data, int old_size, int new_size, short has_ver)
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

    for (int i = 0; i < old_data->num_pieces; ++i) {
        buffer *old_buf = &old_data->pieces[i + has_ver].buf;
        buffer *new_buf = &new_data->pieces[i + 1].buf;

        buffer_reset(old_buf);
        buffer_reset(new_buf);

        if (i < 6) {
            int bytesize = piece_bytesize[i];
            migrate_buffer_mapsize(new_buf, old_buf, bytesize, old_size, new_size);
        } else {
            buffer_write_raw(new_buf, old_buf->data, old_buf->size);
        }

        buffer_reset(old_buf);
        buffer_reset(new_buf);
    }
}

int migrate_scenario_and_load_from_state(scenario_data *migrated_data, scenario_data *data, int version)
{
    switch (version) {
        case 0: // classic maps are 162x162, have no version buffer
            log_info("Migrating legacy map", 0, 0);
            migrate_scenario_mapsize(migrated_data, data, 162, GRID_SIZE, 0);
            migrate_scenario_map_data(migrated_data->state.scenario, data->state.scenario, 162, GRID_SIZE);
            break;
        default:
            return 0; // unsupported scenario version
    }

    for (int i = 0; i < migrated_data->num_pieces; ++i) {
        buffer_reset(&migrated_data->pieces[i].buf);
    }

    return 1;
}

void migrate_savegame_mapsize(savegame_data *new_data, savegame_data *old_data, int old_size, int new_size)
{
    // only needed for the map-related buffers
    int piece_bytesize[] = {
        1, 1, // mission and file_version, skipped
        2, 1, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1
    };

    for (int i = 0; i < old_data->num_pieces; ++i) {
        buffer *old_buf = &old_data->pieces[i].buf;
        buffer *new_buf = &new_data->pieces[i].buf;

        buffer_reset(old_buf);
        buffer_reset(new_buf);

        if (i > 1 && i < 16) {
            migrate_buffer_mapsize(new_buf, old_buf, piece_bytesize[i], old_size, new_size);
        } else { //if (i > 20 && i != 25 && (i < 57 || i > 59)) { // skip figures/routes/buildings
            buffer_write_raw(new_buf, old_buf->data, old_buf->size);
        }

        buffer_reset(old_buf);
        buffer_reset(new_buf);
    }
}

int migrate_savegame_and_load_from_state(savegame_data *migrated_data, savegame_data *data, int version)
{
    switch (version) {
        case SAVE_GAME_VERSION_LEGACY: // classic maps are 162x162, have no version buffer
        case SAVE_GAME_VERSION_AUG_V1:  // same for the initial augustus expanded save version
            log_info("Migrating legacy map size", 0, 0);
            migrate_savegame_mapsize(migrated_data, data, 162, GRID_SIZE);
            migrate_scenario_map_data(migrated_data->state.scenario, data->state.scenario, 162, GRID_SIZE);
            //migrate_figure_data(&migrated_data, data);
            break;
        default:
            return 0; // unsupported savegame version            
    }

    for (int i = 0; i < migrated_data->num_pieces; ++i) {
        buffer_reset(&migrated_data->pieces[i].buf);
    }

    return 1;
}
