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

typedef enum {
    TRANSLATE_NONE,
    TRANSLATE_X,
    TRANSLATE_Y,
    TRANSLATE_OFFSET
} translate_instruction;

typedef struct {
    int offset;
    int old_size;
    int new_size;
    translate_instruction translation;
    int sign;
} change_item;

static int old_grid_border_size;

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

static int64_t translate_old_xy_value(int64_t value, translate_instruction instruction)
{
    if (value == 0) { return value; }

    switch (instruction) {
        case TRANSLATE_X:
            return value - (old_grid_border_size / 2) + (scenario.map.grid_border_size / 2);
        case TRANSLATE_Y:
            return value - old_grid_border_size + scenario.map.grid_border_size;
        case TRANSLATE_OFFSET: {
            int x = value % (scenario.map.width + (int64_t)old_grid_border_size);
            int y = value / (scenario.map.width + (int64_t)old_grid_border_size);
            return ((y + (int64_t)scenario.map.grid_border_size) * (int64_t)scenario.map.width) + (x + (int64_t)scenario.map.grid_border_size);
        }
    }
    return 0;
}

static void migrate_scenario_map_camera_data(buffer *new_scenario, buffer *old_scenario,
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
    old_grid_border_size = scenario.map.grid_border_size;

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
    change_item *changeset, int obj_size, int obj_count, int initial_offset)
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
            int offset_curr = changeset[idx_offset].offset;
            
            int length = offset_curr - offset_prev;

            // first copy the data up to the offset
            buffer_write_raw(new_buf, old_buf->data + offset_into_all_obj + offset_prev, length);

            // then read the old data into a temporary
            uint64_t value = 0;
            buffer_set(old_buf, offset_into_all_obj + offset_prev + length);
            buffer_read_raw(old_buf, &value, changeset[idx_offset].old_size);

            // translate the value to the new xy space, if needed
            value = translate_old_xy_value(value, changeset[idx_offset].translation);

            // write translated value into new buffer
            buffer_write_raw(new_buf, &value, changeset[idx_offset].new_size);

            offset_prev = offset_curr + changeset[idx_offset].old_size;
        }
        // copy remainder from last offset to end of object
        buffer_write_raw(new_buf, old_buf->data + offset_into_all_obj + offset_prev,
            obj_size - offset_prev);
    }
    
    // lastly copy any data after the array portion
    int offset_post = initial_offset + (obj_count * obj_size);
    buffer_write_raw(new_buf, old_buf->data + offset_post, old_buf->size - offset_post);
    buffer_reset(old_buf);
}

void migration_strategy_savegame_classic(savegame_data *migrated_data, savegame_data *data, int version)
{
    log_info("Migrating legacy map size", 0, 0);
    // naively copy raw buffers, then go back and spot-update buffers we know need fixing
    copy_raw_buffers(migrated_data->pieces, data->pieces, data->num_pieces);
    migrate_scenario_map_camera_data(migrated_data->state.scenario, data->state.scenario,
        migrated_data->state.city_view_camera, data->state.city_view_camera, 162, GRID_SIZE);
    migrate_savegame_mapsize(migrated_data, data, 162, GRID_SIZE);

    // fixes needed/piece number:

    // figure       16
    {
        change_item changeset[] = {
            { 21, 1, 2, TRANSLATE_X, 0 },
            { 22, 1, 2, TRANSLATE_Y, 0 },
            { 23, 1, 2, TRANSLATE_X, 0 },
            { 24, 1, 2, TRANSLATE_Y, 0 },
            { 27, 2, 4, TRANSLATE_OFFSET, 0 },
            { 29, 1, 2, TRANSLATE_X, 0 },
            { 30, 1, 2, TRANSLATE_Y, 0 },
            { 31, 2, 4, TRANSLATE_OFFSET, 0 },
            { 33, 1, 2, TRANSLATE_X, 0 },
            { 34, 1, 2, TRANSLATE_Y, 0 },
            { 35, 1, 2, TRANSLATE_X, 0 },
            { 36, 1, 2, TRANSLATE_Y, 0 },
        };
        int change_count = 12;

        int old_obj_size = 128;
        int item_count = 1000;
        int initial_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[16].buf, &data->pieces[16].buf,
            change_count, changeset, old_obj_size, item_count, initial_offset);
    }

    // formations   19
    {
        int arr_offsets[] = {
            47, 48, 49, 50, 51, 52, 53, 54, 101, 102
        };
        int arr_old_bytes[] = {
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1
        };
        int arr_new_bytes[] = {
            2, 2, 2, 2, 2, 2, 2, 2, 2, 2
        };
        translate_instruction arr_translate[] = {
            TRANSLATE_X, TRANSLATE_Y,
            TRANSLATE_X, TRANSLATE_Y,
            TRANSLATE_X, TRANSLATE_Y,
            TRANSLATE_X, TRANSLATE_Y,
            TRANSLATE_X, TRANSLATE_Y,
        };
        change_item changeset[] = {
            {  47, 1, 2, TRANSLATE_X, 0 },
            {  48, 1, 2, TRANSLATE_Y, 0 },
            {  49, 1, 2, TRANSLATE_X, 0 },
            {  50, 1, 2, TRANSLATE_Y, 0 },
            {  51, 1, 2, TRANSLATE_X, 0 },
            {  52, 1, 2, TRANSLATE_Y, 0 },
            {  53, 1, 2, TRANSLATE_X, 0 },
            {  54, 1, 2, TRANSLATE_Y, 0 },
            { 101, 1, 2, TRANSLATE_X, 0 },
            { 102, 1, 2, TRANSLATE_Y, 0 },
        };
        int change_count = 10;

        int old_obj_size = 128;
        int item_count = 50;
        int global_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[19].buf, &data->pieces[19].buf,
            change_count, changeset, old_obj_size, item_count, global_offset);
    }

    // city_data    21
    {
        change_item changeset[] = {
            { 28172, 1, 2, TRANSLATE_X, 0 },
            { 28173, 1, 2, TRANSLATE_Y, 0 },
            { 28174, 2, 4, TRANSLATE_OFFSET, 0 },
            { 28176, 1, 2, TRANSLATE_X, 0 },
            { 28177, 1, 2, TRANSLATE_Y, 0 },
            { 28178, 2, 4, TRANSLATE_OFFSET, 0 },
            { 28180, 1, 2, TRANSLATE_X, 0 },
            { 28181, 1, 2, TRANSLATE_Y, 0 },
            { 28182, 2, 4, TRANSLATE_OFFSET, 0 },
            { 35264, 1, 2, TRANSLATE_X, 0 },
            { 35265, 1, 2, TRANSLATE_Y, 0 },
            { 35266, 2, 4, TRANSLATE_OFFSET, 0 },
            { 35597, 1, 2, TRANSLATE_X, 0 },
            { 35598, 1, 2, TRANSLATE_Y, 0 },
            { 35599, 2, 4, TRANSLATE_OFFSET, 0 },
        };
        int change_count = 15;

        int old_obj_size = 36136;
        int item_count = 1;
        int initial_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[21].buf, &data->pieces[21].buf,
            change_count, changeset, old_obj_size, item_count, initial_offset);
    }

    // building     25
    {
        change_item changeset[] = {
            {  6, 1, 2, TRANSLATE_X, 0 },
            {  7, 1, 2, TRANSLATE_Y, 0 },
            {  9, 2, 4, TRANSLATE_OFFSET, 0 },
            { 32, 1, 2, TRANSLATE_X, 0 },
            { 33, 1, 2, TRANSLATE_Y, 0 },
        };
        int change_count = 5;

        int old_obj_size = 128;
        int item_count = 2000;
        int initial_offset = 0;
        insert_offsets_with_array(&migrated_data->pieces[25].buf, &data->pieces[25].buf,
            change_count, changeset, old_obj_size, item_count, initial_offset);
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
