#include "city.h"

#include "building/menu.h"
#include "city/finance.h"
#include "city/population.h"
#include "game/time.h"
#include "graphics/generic_button.h"
#include "graphics/graphics.h"
#include "graphics/image.h"
#include "graphics/image_button.h"
#include "graphics/lang_text.h"
#include "graphics/screen.h"
#include "graphics/text.h"
#include "input/scroll.h"
#include "map/grid.h"
#include "widget/city.h"
#include "widget/minimap.h"
#include "window/build_menu.h"

static void button_build(int submenu, int param2);

const int buttons_width = 52;
const int buttons_height = 80;
int buttons_offset_x = 0;
int buttons_offset_y = 0;

const int offset_funds = 0;
const int offset_population = 52 * 12 / 2 - 60;
const int offset_date = 52 * 12 - 120;

static image_button buttons_build[] = {
    {52 * 0 , 0, 52, 80, IB_NORMAL, GROUP_SIDEBAR_BUTTONS, 0,  button_build, button_none, BUILD_MENU_VACANT_HOUSE,   0, 1},
    {52 * 1 , 0, 52, 80, IB_NORMAL, GROUP_SIDEBAR_BUTTONS, 8,  button_build, button_none, BUILD_MENU_CLEAR_LAND,     0, 1},
    {52 * 2 , 0, 52, 80, IB_NORMAL, GROUP_SIDEBAR_BUTTONS, 12, button_build, button_none, BUILD_MENU_ROAD,           0, 1},
    {52 * 3 , 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 4,  button_build, button_none, BUILD_MENU_WATER,          0, 1},
    {52 * 4 , 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 40, button_build, button_none, BUILD_MENU_HEALTH,         0, 1},
    {52 * 5 , 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 28, button_build, button_none, BUILD_MENU_TEMPLES,        0, 1},
    {52 * 6 , 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 24, button_build, button_none, BUILD_MENU_EDUCATION,      0, 1},
    {52 * 7 , 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 20, button_build, button_none, BUILD_MENU_ENTERTAINMENT,  0, 1},
    {52 * 8 , 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 16, button_build, button_none, BUILD_MENU_ADMINISTRATION, 0, 1},
    {52 * 9 , 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 44, button_build, button_none, BUILD_MENU_ENGINEERING,    0, 1},
    {52 * 10, 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 36, button_build, button_none, BUILD_MENU_SECURITY,       0, 1},
    {52 * 11, 0, 52, 80, IB_BUILD,  GROUP_SIDEBAR_BUTTONS, 32, button_build, button_none, BUILD_MENU_INDUSTRY,       0, 1},
};

static struct {
    int focus_button_for_tooltip;
    int first_focus;
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

static void calculate_offsets(void)
{
    buttons_offset_x = (screen_width() / 2) - ((buttons_width * 12) / 2);
    buttons_offset_y = screen_height() - buttons_height;
}

void draw_info_bar(void)
{
    int image_base = image_group(GROUP_TOP_MENU_SIDEBAR);
    int blocks_wide = (buttons_width * 12) / 16;
    int top_bar_y_offset = buttons_offset_y - 24;

    graphics_set_clip_rectangle(buttons_offset_x, top_bar_y_offset, buttons_width * 12, 24);

    for (int i = 0; i < blocks_wide; ++i) {
        image_draw(image_base + i % 8, buttons_offset_x + i * 24, top_bar_y_offset);
    }
    image_draw(image_base + 14, buttons_offset_x + offset_funds, top_bar_y_offset);
    image_draw(image_base + 14, buttons_offset_x + offset_population, top_bar_y_offset);
    image_draw(image_base + 14, buttons_offset_x + offset_date, top_bar_y_offset);

    color_t treasury_color = COLOR_WHITE;
    int treasury = city_finance_treasury();
    if (treasury < 0) {
        treasury_color = COLOR_FONT_RED;
    }
    int width = lang_text_draw_colored(6, 0, buttons_offset_x + offset_funds + 12, top_bar_y_offset + 5, FONT_NORMAL_PLAIN, treasury_color);
    text_draw_number_colored(treasury, '@', " ", buttons_offset_x + 8 + width, top_bar_y_offset + 5, FONT_NORMAL_PLAIN, treasury_color);

    width = lang_text_draw_colored(6, 1, buttons_offset_x + offset_population + 12, top_bar_y_offset + 5, FONT_NORMAL_PLAIN, COLOR_WHITE);
    text_draw_number_colored(city_population(), '@', " ", buttons_offset_x + offset_population + 8 + width, top_bar_y_offset + 5, FONT_NORMAL_PLAIN, COLOR_WHITE);

    lang_text_draw_month_year_max_width(game_time_month(), game_time_year(), buttons_offset_x + offset_date + 12, top_bar_y_offset + 5, 100, FONT_NORMAL_PLAIN, COLOR_FONT_YELLOW);

    graphics_reset_clip_rectangle();
}

void draw_button_bar(void)
{
    graphics_set_clip_rectangle(buttons_offset_x, buttons_offset_y, buttons_width * 12, buttons_height);
    for (int i = 0; i < 12; ++i) {
        int offset_x = buttons_offset_x + i * buttons_width;
        image_draw(image_group(GROUP_EMPIRE_PANELS) + 3, offset_x, buttons_offset_y);
        image_draw(image_group(GROUP_EMPIRE_PANELS) + 3, offset_x, buttons_offset_y + 32);
        image_draw(image_group(GROUP_EMPIRE_PANELS) + 3, offset_x, buttons_offset_y + 64);

        int focus = (data.focus_button_for_tooltip - 19 == i + 1);
        button_border_draw(buttons_offset_x + i * buttons_width, buttons_offset_y, buttons_width, buttons_height, focus);
    }
    graphics_reset_clip_rectangle();

    image_buttons_draw(buttons_offset_x + 7, buttons_offset_y + 7, buttons_build, 12);
}

void draw_minimap(void)
{
    //image_draw(image_group(GROUP_SIDE_PANEL) + 4, 0, screen_height() - 160);
    //image_draw(image_group(GROUP_SIDE_PANEL) + 4, 158, screen_height() - 160);

    // 160 being the largest map size possible
    int map_offset_x = 160 - map_grid_width();
    int map_offset_y = screen_height() - 160 - map_grid_height();
    widget_minimap_invalidate();
    widget_minimap_draw(map_offset_x, map_offset_y, map_grid_width(), map_grid_height() * 2, 0);
}

void widget_octavius_ui_city_draw_foreground(void)
{
    calculate_offsets();

    if (building_menu_has_changed()) {
        enable_building_buttons();
    }

    draw_info_bar();

    draw_button_bar();

    draw_minimap();
}

void widget_octavius_ui_city_draw_foreground_military(void)
{

}

int widget_octavius_ui_city_handle_mouse(const mouse *m)
{
    int handled = 0;
    int button_id = 0;
    data.focus_button_for_tooltip = 0;
    data.focus_button_for_tooltip = 0;

    if (widget_minimap_handle_mouse(m)) {
        return 1;
    }

    handled = image_buttons_handle_mouse(m, buttons_offset_x, buttons_offset_y, buttons_build, 12, &button_id);
    if (button_id) {
        data.focus_button_for_tooltip = button_id + 19;
        data.first_focus = data.focus_button_for_tooltip;
    }

    return ((m->left.is_down || m->left.went_down || m->left.went_up) ? handled : 0);
}

int widget_octavius_ui_city_handle_mouse_build_menu(const mouse *m)
{
    int handled = 0;
    int button_id = 0;
    int up = m->left.went_up;
    handled = image_buttons_handle_mouse(m, buttons_offset_x, buttons_offset_y, buttons_build, 12, &button_id);

    if (handled && m->left.is_down) {
        if (button_id) {
            data.focus_button_for_tooltip = button_id + 19;
        }
    } else if (handled && up) {
        if (button_id) {
            data.focus_button_for_tooltip = button_id + 19;
            data.first_focus = data.focus_button_for_tooltip;
        } else {
            data.focus_button_for_tooltip = data.first_focus;
        }
    } else if (!handled && up) {
        data.focus_button_for_tooltip = data.first_focus;
    }
    return handled;
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
