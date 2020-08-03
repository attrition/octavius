#include "save_data.h"

#include <stdlib.h>
#include <string.h>

void init_file_piece(file_piece *piece, int size, int compressed)
{
    piece->compressed = compressed;
    void *data = malloc(size);
    memset(data, 0, size);
    buffer_init(&piece->buf, data, size);
}

buffer *save_data_create_scenario_piece(scenario_data *data, int size)
{
    file_piece *piece = &data->pieces[data->num_pieces++];
    init_file_piece(piece, size, 0);
    return &piece->buf;
}

buffer *save_data_create_savegame_piece(savegame_data *data, int size, int compressed)
{
    file_piece *piece = &data->pieces[data->num_pieces++];
    init_file_piece(piece, size, compressed);
    return &piece->buf;
}