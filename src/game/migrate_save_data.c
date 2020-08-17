#include "migrate_save_data.h"

#include "building/building.h"
#include "city/view.h"
#include "core/buffer.h"
#include "core/log.h"
#include "figure/figure.h"
#include "figure/formation.h"
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

void migrate_scenario_map_camera_data(buffer *new_scenario, buffer *old_scenario,
    buffer *new_camera, buffer *old_camera, int old_size, int new_size)
{
    // prep buffers for reading/writing
    buffer_reset(old_camera);
    buffer_reset(old_scenario);
    buffer_reset(new_camera);
    buffer_reset(new_scenario);

    // store resized information in original state
    // fetch incoming map data from old_data
    int width, height;
    scenario_load_state(old_scenario);
    scenario_map_init();
    map_grid_size(&width, &height);

    // fetch existing camera offset
    city_view_load_scenario_state(old_camera);
    int oldx, oldy;
    city_view_get_camera(&oldx, &oldy);
    oldx -= scenario.map.grid_border_size / 2;
    oldy -= scenario.map.grid_border_size;

    // store modified map data back into new_data
    scenario.map.grid_border_size = new_size - scenario.map.width;
    scenario.map.grid_start = (new_size - scenario.map.height) / 2 *
        new_size + (new_size - scenario.map.width) / 2;
    scenario_save_state(new_scenario);

    // apply new map boundary to previous camera offset
    int newx, newy;
    newx = oldx + scenario.map.grid_border_size / 2;
    newy = oldy + scenario.map.grid_border_size;
    city_view_set_camera(newx, newy);
    city_view_save_scenario_state(new_camera);
}

void migrate_scenario_mapsize(scenario_data *new_data, scenario_data *old_data,
    int old_size, int new_size, short has_ver)
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

        if (i < 6) {
            int bytesize = piece_bytesize[i];
            migrate_buffer_mapsize(new_buf, old_buf, bytesize, old_size, new_size);
        } else {
            buffer_reset(new_buf);
            buffer_write_raw(new_buf, old_buf->data, old_buf->size);
        }
    }
}

int migrate_scenario_data(scenario_data *migrated_data, scenario_data *data, int version)
{
    switch (version) {
        case 0: // classic maps are 162x162, have no version buffer
            log_info("Migrating legacy map", 0, 0);
            migrate_scenario_mapsize(migrated_data, data, 162, GRID_SIZE, 0);
            migrate_scenario_map_camera_data(migrated_data->state.scenario, data->state.scenario,
                migrated_data->state.camera, data->state.camera, 162, GRID_SIZE);
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
        2, 1, 2, 2, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1
    };

    for (int i = 2; i < 16; ++i) {
        buffer *old_buf = &old_data->pieces[i].buf;
        buffer *new_buf = &new_data->pieces[i].buf;

        migrate_buffer_mapsize(new_buf, old_buf, piece_bytesize[i - 2], old_size, new_size);
    }
}

void copy_raw_buffers(file_piece *new_pieces, file_piece *old_pieces, int num_pieces)
{
    for (int i = 0; i < num_pieces; ++i) {
        buffer *old_buf = &old_pieces[i].buf;
        buffer *new_buf = &new_pieces[i].buf;

        buffer_reset(new_buf); // defensive reset
        buffer_write_raw(new_buf, old_buf->data, old_buf->size);
    }
}

void insert_offsets_with_array(buffer *new_buf, buffer *old_buf, int offset_count,
    int offsets[], int insert_bytes[], int obj_size, int obj_count, int initial_offset)
{
    int padding = 0;
    buffer_reset(new_buf);

    // there may be data before the array portion begins
    if (initial_offset) {
        buffer_write_raw(new_buf, old_buf->data, initial_offset);
    }

    for (int idx_object = 0; idx_object < obj_count; ++idx_object) {
        int offset_prev = 0;
        int offset_into_all_obj = initial_offset + (idx_object * obj_size);
        for (int idx_offset = 0; idx_offset < offset_count; ++idx_offset) {
            int offset_curr = offsets[idx_offset];
            
            int length = offset_curr - offset_prev;

            // first copy the data up to the offset
            buffer_write_raw(new_buf, old_buf->data + offset_into_all_obj + offset_prev, length);
            // then insert the padding
            buffer_write_raw(new_buf, &padding, insert_bytes[idx_offset]);

            offset_prev = offset_curr;
        }
        // copy remainder from last offset to end of object
        buffer_write_raw(new_buf, old_buf->data + offset_into_all_obj + offset_prev,
            obj_size - offset_prev);
    }
    
    // lastly copy any data after the array portion
    int offset_post = initial_offset + (obj_count * obj_size);
    buffer_write_raw(new_buf, old_buf->data + offset_post, old_buf->size - offset_post);
}

void migration_strategy_savegame_classic(savegame_data *migrated_data, savegame_data *data, int version)
{
    log_info("Migrating legacy map size", 0, 0);
    // naively copy raw buffers, then go back and spot-update buffers we know need fixing
    copy_raw_buffers(migrated_data->pieces, data->pieces, data->num_pieces);
    migrate_savegame_mapsize(migrated_data, data, 162, GRID_SIZE);
    migrate_scenario_map_camera_data(migrated_data->state.scenario, data->state.scenario,
        migrated_data->state.city_view_camera, data->state.city_view_camera, 162, GRID_SIZE);

    // fixes needed/piece number:

    // figure       16
    {
        int arr_offsets[] = { // where to punch in extra bytes
            21, 22, 23, 24, 27, 29,
            30, 31, 33, 34, 35, 36
        };
        int arr_insert_bytes[] = { // how many bytes to punch in, parallel to arr_offsets
            1, 1, 1, 1, 2, 1, 1, 2, 1, 1, 1, 1
        };
        int arr_count = 12;

        int old_obj_size = 128;
        int item_count = 1000;
        int initial_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[16].buf, &data->pieces[16].buf,
            arr_count, arr_offsets, arr_insert_bytes, old_obj_size, item_count, initial_offset);
    }

    // formations   19
    {
        int arr_offsets[] = {
            47, 48, 49, 50, 51, 52, 53, 54, 101, 102
        };
        int arr_insert_bytes[] = {
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1
        };
        int arr_count = 10;

        int old_obj_size = 128;
        int item_count = 50;
        int global_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[19].buf, &data->pieces[19].buf,
            arr_count, arr_offsets, arr_insert_bytes, old_obj_size, item_count, global_offset);
    }

    // city_data    21
    {
        int arr_offsets[] = {
            28172, 28173, 28174, // entry point
            28176, 28177, 28178, // exit point
            28180, 28181, 28182, // senate
            35264, 35265, 35266, // barracks
            35597, 35598, 35599  // distribution center
        };
        int arr_insert_bytes[] = {
            1, 1, 2, 1, 1, 2, 1, 1, 2,
            1, 1, 2, 1, 1, 2
        };
        int arr_count = 15;

        int old_obj_size = 36136;
        int item_count = 1;
        int initial_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[21].buf, &data->pieces[21].buf,
            arr_count, arr_offsets, arr_insert_bytes, old_obj_size, item_count, initial_offset);
    }

    // building     25
    {
        int arr_offsets[] = { 6, 7, 9, 32, 33 };
        int arr_insert_bytes[] = { 1, 1, 2, 1, 1 };
        int arr_count = 5;

        int old_obj_size = 128;
        int item_count = 2000;
        int initial_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[25].buf, &data->pieces[25].buf,
            arr_count, arr_offsets, arr_insert_bytes, old_obj_size, item_count, initial_offset);
    }
}

int migrate_savegame_data(savegame_data *migrated_data, savegame_data *data, int version)
{
    switch (version) {
        case SAVE_GAME_VERSION_LEGACY: // classic maps are 162x162, have no version buffer
        case SAVE_GAME_VERSION_AUG_V1: // same for the initial augustus expanded save version
            migration_strategy_savegame_classic(migrated_data, data, version);
            break;
        default:
            return 0; // unsupported savegame version            
    }

    for (int i = 0; i < migrated_data->num_pieces; ++i) {
        buffer_reset(&migrated_data->pieces[i].buf);
    }

    return 1;
}
