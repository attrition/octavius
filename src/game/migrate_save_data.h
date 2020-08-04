#ifndef GAME_MIGRATE_SAVE_DATA_H
#define GAME_MIGRATE_SAVE_DATA_H

#include "save_data.h"

int migrate_savegame_and_load_from_state(savegame_data *migrated_data, savegame_data *data, int version);

int migrate_scenario_and_load_from_state(scenario_data *migrated_data, scenario_data *data, int version);

#endif
