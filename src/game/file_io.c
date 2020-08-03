#include "file_io.h"

#include "core/file.h"
#include "core/log.h"
#include "game/migrate_save_data.h"
#include "game/save_data.h"
#include "core/zip.h"

#include <stdio.h>
#include <stdint.h>

#define COMPRESS_BUFFER_SIZE 3000000
#define UNCOMPRESSED 0x80000000

static char compress_buffer[COMPRESS_BUFFER_SIZE];

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
            save_data_init_scenario_data_legacy(&data);
            break;
        case 1:
            log_info("Loading Augustus scenario, version", 0, scenario_ver);
            save_data_init_scenario_data_current(&data);
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
        save_data_init_scenario_data_current(&migrated_data);
        if (!migrate_scenario_and_load_from_state(&migrated_data, &data, scenario_ver)) {
            log_error("Failed to migrate and load scenario current version", 0, 0);
            return 0;
        }
        save_data_scenario_load_from_state(&migrated_data.state, scenario_ver);
        return 1;
    }

    save_data_scenario_load_from_state(&data.state, scenario_ver);
    return 1;
}

int game_file_io_write_scenario(const char *filename)
{
    scenario_data data = {0};

    log_info("Saving scenario", filename, 0);

    save_data_init_scenario_data_current(&data);
    save_data_scenario_save_to_state(&data.state);

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
        case SAVE_GAME_VERSION_LEGACY:
            log_info("Loading saved game with legacy format.", 0, 0);
            save_data_init_savegame_data_legacy(&data);
            break;
        case SAVE_GAME_VERSION_AUG_V1:
        case SAVE_GAME_VERSION:
            log_info("Loading saved game with augustus current format.", 0, 0);
            save_data_init_savegame_data_augustus(&data, savegame_version);
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
        save_data_init_savegame_data_augustus(&migrated_data, SAVE_GAME_VERSION);
        if (!migrate_savegame_and_load_from_state(&migrated_data, &data, savegame_version)) {
            log_error("Failed to migrate and load savegame current version", 0, 0);
            return 0;
        }
        save_data_savegame_load_from_state(&migrated_data.state);
        return 1;
    }

    save_data_savegame_load_from_state(&data.state);
    return 1;
}

int game_file_io_write_saved_game(const char *filename)
{
    savegame_data data = {0};
    log_info("Saving game", filename, 0);
    int savegame_version = SAVE_GAME_VERSION;

    save_data_init_savegame_data_augustus(&data, savegame_version);
    save_data_savegame_save_to_state(&data.state, savegame_version);

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
