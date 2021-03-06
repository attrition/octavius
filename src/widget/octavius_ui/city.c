#include "city.h"

#include "building/menu.h"
#include "graphics/image.h"
#include "graphics/image_button.h"
#include "graphics/screen.h"
#include "input/scroll.h"
#include "map/grid.h"
#include "widget/city.h"
#include "widget/minimap.h"
#include "window/build_menu.h"

static void button_build(int submenu, int param2);

const int buttons_width = 39;
const int buttons_height = 26;
int buttons_x_offset = 0;
int buttons_y_offset = 0;

static image_button buttons_build[] = {
    
    {39 * 0,  0, 39, 26, IB_NORMAL, GROUP_SIDEBAR_BUTTONS, 0,  button_build, button_none, BUILD_MENU_VACANT_HOUSE, 0, 1},
    {39 * 1,  0, 39, 26, IB_NORMAL, GROUP_SIDEBAR_BUTTONS, 8,  button_build, button_none, BUILD_MENU_CLEAR_LAND, 0, 1},
    {39 * 2,  0, 39, 26, IB_NORMAL, GROUP_SIDEBAR_BUTTONS, 12, button_build, button_none, BUILD_MENU_ROAD, 0, 1},
    {39 * 3,  0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 4,  button_build, button_none, BUILD_MENU_WATER, 0, 1},
    {39 * 4,  0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 40, button_build, button_none, BUILD_MENU_HEALTH, 0, 1},
    {39 * 5,  0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 28, button_build, button_none, BUILD_MENU_TEMPLES, 0, 1},
    {39 * 6,  0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 24, button_build, button_none, BUILD_MENU_EDUCATION, 0, 1},
    {39 * 7,  0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 20, button_build, button_none, BUILD_MENU_ENTERTAINMENT, 0, 1},
    {39 * 8,  0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 16, button_build, button_none, BUILD_MENU_ADMINISTRATION, 0, 1},
    {39 * 9,  0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 44, button_build, button_none, BUILD_MENU_ENGINEERING, 0, 1},
    {39 * 10, 0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 36, button_build, button_none, BUILD_MENU_SECURITY, 0, 1},
    {39 * 11, 0, 39, 26, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 32, button_build, button_none, BUILD_MENU_INDUSTRY, 0, 1},
};

static struct {
    int focus_button_for_tooltip;
} data;

void widget_octavius_ui_city_draw_background(void)
{

}

static void enable_building_buttons(void)
{
    for (int i = 0; i < 12; i++) {
        buttons_build[i].enabled = 1;
        if (building_menu_count_items(buttons_build[i].parameter1) <= 0) {
            buttons_build[i].enabled = 0;
        }
    }
}

static void calculate_offsets()
{
    buttons_x_offset = (screen_width() / 2) - ((buttons_width * 12) / 2);
    buttons_y_offset = screen_height() - buttons_height;
}

void widget_octavius_ui_city_draw_foreground(void)
{
    calculate_offsets();

    if (building_menu_has_changed()) {
        enable_building_buttons();
    }

    if (scroll_in_progress()) {
        widget_minimap_invalidate();
    }

    image_buttons_draw(buttons_x_offset, buttons_y_offset, buttons_build, 12);

    int map_height = map_grid_height() * 2;
    widget_minimap_draw(0, screen_height() - map_height, map_grid_width(), map_height, 1);
}

void widget_octavius_ui_city_draw_foreground_military(void)
{

}

int widget_octavius_ui_city_handle_mouse(const mouse *m)
{
    int handled = 0;
    int button_id;
    data.focus_button_for_tooltip = 0;

    if (widget_minimap_handle_mouse(m)) {
        return 1;
    }

    handled |= image_buttons_handle_mouse(m, buttons_x_offset, buttons_y_offset, buttons_build, 12, &button_id);
    if (button_id) {
        data.focus_button_for_tooltip = button_id + 19;
    }

    return handled;
}

int widget_octavius_ui_city_handle_mouse_build_menu(const mouse *m)
{
    return image_buttons_handle_mouse(m, buttons_x_offset, buttons_y_offset, buttons_build, 12, 0);
}

int widget_octavius_ui_city_get_tooltip_text(void)
{
    return data.focus_button_for_tooltip;
}

// --------

static void button_build(int submenu, int param2)
{
    window_build_menu_show(submenu);
}
