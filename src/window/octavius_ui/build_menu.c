#include "build_menu.h"

#include "building/construction.h"
#include "building/menu.h"
#include "building/model.h"
#include "city/view.h"
#include "core/config.h"
#include "core/string.h"
#include "game/resource.h"
#include "graphics/generic_button.h"
#include "graphics/graphics.h"
#include "graphics/image.h"
#include "graphics/lang_text.h"
#include "graphics/octavius_ui/build_button.h"
#include "graphics/panel.h"
#include "graphics/screen.h"
#include "graphics/text.h"
#include "graphics/window.h"
#include "input/input.h"
#include "scenario/property.h"
#include "widget/city.h"
#include "widget/octavius_ui/city.h"
#include "window/city.h"

static void build_button_menu_index(int param1, int param2, int param3);
static void generic_button_menu_index(int param1, int param2);
static void button_menu_item(int submenu, int item);

typedef struct {
    int offset_x;
    int offset_y;
    int group_index;
    building_type building_type;
} submenu_button_details;

typedef struct {
    int button_count;
    build_button *buttons;
    int offset_x;
    int offset_y;
    int width;
    int height;
    submenu_button_details *button_details;
} menu_definition;

// param1: submenu
// param2: item index in submenu
// param3: calculated actual item index
static build_button build_menu_water_buttons[] = {
    { 0,   0, 60, 100, IB_NORMAL, 0, 0, build_button_menu_index, build_button_none, 3, 1, 1, 1 },
    { 60,  0, 60, 100, IB_NORMAL, 0, 0, build_button_menu_index, build_button_none, 3, 2, 1, 1 },
    { 120, 0, 60, 100, IB_NORMAL, 0, 0, build_button_menu_index, build_button_none, 3, 3, 1, 1 },
    { 180, 0, 60, 100, IB_NORMAL, 0, 0, build_button_menu_index, build_button_none, 3, 4, 1, 1 },
};

static submenu_button_details build_menu_water_definitions[] = {
    { -40, 30, GROUP_BUILDING_RESERVOIR,  BUILDING_RESERVOIR },
    {   0, 50, GROUP_BUILDING_AQUEDUCT,   BUILDING_AQUEDUCT  },
    {   0, 50, GROUP_BUILDING_FOUNTAIN_1, BUILDING_FOUNTAIN  },
    {   0, 50, GROUP_BUILDING_WELL,       BUILDING_WELL      },
};

static menu_definition menu_definitions[] = {
    { 4, build_menu_water_buttons, -3, -3, 246, 106, build_menu_water_definitions }
};

static generic_button build_menu_buttons[] = {
    {0, 0, 256, 20, generic_button_menu_index, button_none, 1, 0},
    {0, 24, 256, 20, generic_button_menu_index, button_none, 2, 0},
    {0, 48, 256, 20, generic_button_menu_index, button_none, 3, 0},
    {0, 72, 256, 20, generic_button_menu_index, button_none, 4, 0},
    {0, 96, 256, 20, generic_button_menu_index, button_none, 5, 0},
    {0, 120, 256, 20, generic_button_menu_index, button_none, 6, 0},
    {0, 144, 256, 20, generic_button_menu_index, button_none, 7, 0},
    {0, 168, 256, 20, generic_button_menu_index, button_none, 8, 0},
    {0, 192, 256, 20, generic_button_menu_index, button_none, 9, 0},
    {0, 216, 256, 20, generic_button_menu_index, button_none, 10, 0},
    {0, 240, 256, 20, generic_button_menu_index, button_none, 11, 0},
    {0, 264, 256, 20, generic_button_menu_index, button_none, 12, 0},
    {0, 288, 256, 20, generic_button_menu_index, button_none, 13, 0},
    {0, 312, 256, 20, generic_button_menu_index, button_none, 14, 0},
    {0, 336, 256, 20, generic_button_menu_index, button_none, 15, 0},
    {0, 360, 256, 20, generic_button_menu_index, button_none, 16, 0},
    {0, 384, 256, 20, generic_button_menu_index, button_none, 17, 0},
    {0, 408, 256, 20, generic_button_menu_index, button_none, 18, 0},
    {0, 432, 256, 20, generic_button_menu_index, button_none, 19, 0},
    {0, 456, 256, 20, generic_button_menu_index, button_none, 20, 0},
    {0, 480, 256, 20, generic_button_menu_index, button_none, 21, 0},
    {0, 504, 256, 20, generic_button_menu_index, button_none, 22, 0},
    {0, 528, 256, 20, generic_button_menu_index, button_none, 23, 0},
    {0, 552, 256, 20, generic_button_menu_index, button_none, 24, 0},
    {0, 576, 256, 20, generic_button_menu_index, button_none, 25, 0},
    {0, 600, 256, 20, generic_button_menu_index, button_none, 26, 0},
    {0, 624, 256, 20, generic_button_menu_index, button_none, 27, 0},
    {0, 648, 256, 20, generic_button_menu_index, button_none, 28, 0},
    {0, 672, 256, 20, generic_button_menu_index, button_none, 29, 0},
    {0, 696, 256, 20, generic_button_menu_index, button_none, 30, 0},
};

static struct {
    build_menu_group selected_submenu;
    int num_items;
    int offset_y;

    int focus_button_id;
} data;

static int init(build_menu_group submenu)
{
    data.selected_submenu = submenu;
    data.num_items = building_menu_count_items(submenu);
    data.offset_y = screen_height() - 250;
    if (submenu == BUILD_MENU_VACANT_HOUSE ||
        submenu == BUILD_MENU_CLEAR_LAND ||
        submenu == BUILD_MENU_ROAD) {
        button_menu_item(submenu, 0);
        return 0;
    } else {
        return 1;
    }
}

static void draw_background(void)
{
    window_city_draw_panels();
}

static int is_all_button(building_type type)
{
    return (type == BUILDING_MENU_SMALL_TEMPLES && data.selected_submenu == BUILD_MENU_SMALL_TEMPLES) ||
        (type == BUILDING_MENU_LARGE_TEMPLES && data.selected_submenu == BUILD_MENU_LARGE_TEMPLES);
}

static int get_parent_submenu(void)
{
    switch (data.selected_submenu) {
        case BUILD_MENU_SMALL_TEMPLES:
        case BUILD_MENU_LARGE_TEMPLES:
            return BUILD_MENU_TEMPLES;
        case BUILD_MENU_FORTS:
            return BUILD_MENU_SECURITY;
        case BUILD_MENU_FARMS:
        case BUILD_MENU_RAW_MATERIALS:
        case BUILD_MENU_WORKSHOPS:
            return BUILD_MENU_INDUSTRY;
    }
    return data.selected_submenu;
}

static int button_index_to_submenu_item(int index)
{
    int item = -1;
    for (int i = 0; i <= index; i++) {
        item = building_menu_next_index(data.selected_submenu, item);
    }
    return item;
}

static void generic_button_menu_index(int param1, int param2)
{
    button_menu_item(data.selected_submenu, button_index_to_submenu_item(param1 - 1));
}

static void build_button_menu_index(int param1, int param2, int param3)
{
    if (param3) {
        button_menu_item(param1, param3 - 1);
    }
}

static int get_build_buttons_index(void)
{
    switch (data.selected_submenu) {
        case 0:
        case 1:
        case 2:  return -1;
        case 3:  return  0;
        default: return -1;
    }
}

static void draw_building(int image_id, int x, int y, int enabled)
{
    int mask = enabled ? 0 : COLOR_RED;
    image_draw_isometric_footprint(image_id, x, y, mask);
    image_draw_isometric_top(image_id, x, y, mask);
}

static void get_menu_offsets(int *x, int *y, int build_buttons_index)
{
    menu_definition *menu = &menu_definitions[build_buttons_index];
    *x = (screen_width() / 2) - ((12 * 52) / 2) + (get_parent_submenu() * 52) - (menu->width / 2) + 26;
    *y = screen_height() - 114 - menu->height;
}

static void draw_build_buttons(void)
{
    int build_buttons_index = get_build_buttons_index();
    menu_definition* menu = &menu_definitions[build_buttons_index];

    int offset_x = 0;
    int offset_y = 0;

    get_menu_offsets(&offset_x, &offset_y, build_buttons_index);
    graphics_fill_rect(offset_x + menu->offset_x, offset_y + menu->offset_y, menu->width, menu->height, COLOR_BLACK);

    for (int i = 0, drawn = 0; i < menu->button_count; ++i, ++drawn) {
        build_button *btn = &menu->buttons[i];
        submenu_button_details *details = &menu->button_details[i];

        int enabled = building_menu_check_index_enabled(btn->parameter1, btn->parameter2 - 1);
        btn->parameter3 = enabled ? btn->parameter2 : 0;

        int start_x = offset_x + btn->x_offset;
        int start_y = offset_y + btn->y_offset;
        graphics_set_clip_rectangle(start_x, start_y, btn->width, btn->height);
        graphics_fill_rect(start_x, start_y, btn->width, btn->height, COLOR_SIDEBAR);

        draw_building(
            image_group(details->group_index),
            start_x + details->offset_x,
            start_y + details->offset_y,
            enabled);
        graphics_draw_inset_rect(start_x, start_y, btn->width, btn->height);

        int type = building_menu_type(btn->parameter1, i);
        if (type == BUILDING_DRAGGABLE_RESERVOIR) {
            type = BUILDING_RESERVOIR;
        }
        int cost = model_get_building(type)->cost;
        if (type == BUILDING_FORT) {
            cost = 0;
        }
        if (type == BUILDING_MENU_SMALL_TEMPLES && btn->parameter1 == BUILD_MENU_SMALL_TEMPLES) {
            cost = model_get_building(BUILDING_SMALL_TEMPLE_CERES)->cost;
        }
        if (type == BUILDING_MENU_LARGE_TEMPLES && btn->parameter1 == BUILD_MENU_LARGE_TEMPLES) {
            cost = model_get_building(BUILDING_LARGE_TEMPLE_CERES)->cost;
        }

        // draw tooltip?
        if (cost) {
            //text_draw_money(cost, x_start, y_start + btn->height - 15, FONT_NORMAL_GREEN);
        }


    }
    graphics_reset_clip_rectangle();
}

static void draw_menu_buttons(void)
{
    if (get_build_buttons_index() != -1) {
        draw_build_buttons();
        return;
    }

    int offset_x = (screen_width() / 2) - ((12 * 52) / 2) + 164 + (get_parent_submenu() * 52);
    int item_index = -1;
    int y_step = -24;

    for (int i = 0; i < data.num_items; i++) {
        item_index = building_menu_next_index(data.selected_submenu, item_index);
        int real_index = data.num_items - i - 1;

        label_draw(offset_x - 266, data.offset_y + 110 + y_step * real_index, 16, data.focus_button_id == i + 1 ? 1 : 2);
        int type = building_menu_type(data.selected_submenu, item_index);
        if (is_all_button(type)) {
            lang_text_draw_centered(52, 19, offset_x - 266, data.offset_y + 113 + y_step * real_index, 176, FONT_NORMAL_GREEN);
        } else {
            lang_text_draw_centered(28, type, offset_x - 266, data.offset_y + 113 + y_step * real_index, 176, FONT_NORMAL_GREEN);
        }
        if (type == BUILDING_DRAGGABLE_RESERVOIR) {
            type = BUILDING_RESERVOIR;
        }
        int cost = model_get_building(type)->cost;
        if (type == BUILDING_FORT) {
            cost = 0;
        }
        if (type == BUILDING_MENU_SMALL_TEMPLES && data.selected_submenu == BUILD_MENU_SMALL_TEMPLES) {
            cost = model_get_building(BUILDING_SMALL_TEMPLE_CERES)->cost;
        }
        if (type == BUILDING_MENU_LARGE_TEMPLES && data.selected_submenu == BUILD_MENU_LARGE_TEMPLES) {
            cost = model_get_building(BUILDING_LARGE_TEMPLE_CERES)->cost;
        }
        if (cost) {
            text_draw_money(cost, offset_x - 82, data.offset_y + 114 + y_step * real_index, FONT_NORMAL_GREEN);
        }
    }
}

static void draw_foreground(void)
{
    window_city_draw_all();
    draw_menu_buttons();
}

static int handle_build_submenu(const mouse *m)
{
    int offset_x = (screen_width() / 2) - ((12 * 52) / 2) + (get_parent_submenu() * 52) - 102;
    int offset_y = data.offset_y + 110 + (data.num_items - 1) * -24;

    int build_buttons_index = get_build_buttons_index();
    if (build_buttons_index != -1) {
        offset_x = 0;
        offset_y = 0;
        get_menu_offsets(&offset_x, &offset_y, build_buttons_index);
        menu_definition *menu = &menu_definitions[build_buttons_index];
        return build_buttons_handle_mouse(
            m, offset_x, offset_y, menu->buttons, menu->button_count, &data.focus_button_id);
    }

    return generic_buttons_handle_mouse(
        m, offset_x, offset_y, build_menu_buttons, data.num_items, &data.focus_button_id);
}

static void handle_input(const mouse *m, const hotkeys *h)
{
    if (handle_build_submenu(m) || widget_octavius_ui_city_handle_mouse_build_menu(m)) {
        return;
    }
    if (input_go_back_requested(m, h)) {
        window_city_show();
        return;
    }
}

static int set_submenu_for_type(building_type type)
{
    build_menu_group current_menu = data.selected_submenu;
    switch (type) {
        case BUILDING_MENU_FARMS:
            data.selected_submenu = BUILD_MENU_FARMS;
            break;
        case BUILDING_MENU_RAW_MATERIALS:
            data.selected_submenu = BUILD_MENU_RAW_MATERIALS;
            break;
        case BUILDING_MENU_WORKSHOPS:
            data.selected_submenu = BUILD_MENU_WORKSHOPS;
            break;
        case BUILDING_MENU_SMALL_TEMPLES:
            data.selected_submenu = BUILD_MENU_SMALL_TEMPLES;
            break;
        case BUILDING_MENU_LARGE_TEMPLES:
            data.selected_submenu = BUILD_MENU_LARGE_TEMPLES;
            break;
        case BUILDING_FORT:
            data.selected_submenu = BUILD_MENU_FORTS;
            break;
        default:
            return 0;
    }
    return current_menu != data.selected_submenu;
}

static void button_menu_item(int submenu, int item)
{
    widget_city_clear_current_tile();

    building_type type = building_menu_type(submenu, item);
    building_construction_set_type(type);

    if (set_submenu_for_type(type)) {
        data.num_items = building_menu_count_items(data.selected_submenu);
        data.offset_y = screen_height() - 250;
        building_construction_clear_type();
    } else {
        window_city_show();
    }
}

void window_octavius_build_menu_show(int submenu)
{
    if (init(submenu)) {
        window_type window = {
            WINDOW_BUILD_MENU,
            draw_background,
            draw_foreground,
            handle_input,
            0
        };
        window_show(&window);
    }
}
