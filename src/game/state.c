#include "state.h"

#include "building/construction.h"
#include "city/victory.h"
#include "city/view.h"
#include "city/warning.h"
#include "core/random.h"
#include "game/undo.h"
#include "map/ring.h"
#include "window/main_menu.h"

static struct {
    int paused;
    int current_overlay;
    int previous_overlay;
} data = {0, OVERLAY_NONE, OVERLAY_NONE};

void game_state_init(void)
{
    city_victory_reset();
    map_ring_init();

    city_view_reset_orientation();
    city_view_set_camera(76, 152);

    random_generate_pool();

    city_warning_clear_all();
}

int game_state_is_paused(void)
{
    return data.paused;
}

void game_state_unpause(void)
{
    data.paused = 0;
}

void game_state_toggle_paused(void)
{
    data.paused = data.paused ? 0 : 1;
}

int game_state_overlay(void)
{
    return data.current_overlay;
}

void game_state_reset_overlay(void)
{
    data.current_overlay = OVERLAY_NONE;
    data.previous_overlay = OVERLAY_NONE;
}

void game_state_toggle_overlay(void)
{
    int tmp = data.previous_overlay;
    data.previous_overlay = data.current_overlay;
    data.current_overlay = tmp;
}

void game_state_set_overlay(int overlay)
{
    if (overlay == OVERLAY_NONE) {
        data.previous_overlay = data.current_overlay;
    } else {
        data.previous_overlay = OVERLAY_NONE;
    }
    data.current_overlay = overlay;
}

void game_state_new_game(void)
{
    building_construction_clear_type();
    game_undo_disable();
    game_state_reset_overlay();
    window_main_menu_show(1);
}