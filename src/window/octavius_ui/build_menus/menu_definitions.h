#ifndef WINDOW_OCTAVIUS_BUILD_MENU_MENU_DEFINITIONS_H
#define WINDOW_OCTAVIUS_BUILD_MENU_MENU_DEFINITIONS_H

#include "building/menu.h"
#include "graphics/octavius_ui/build_button.h"

typedef struct {
    int offset_x;
    int offset_y;
    int group_index;
    int image_offset;
    building_type building_type;
    int use_image_draw;
} submenu_button_details;

typedef struct {
    build_button *buttons;
    submenu_button_details *button_details;
    int offset_x;
    // calculated
    int width;
    int height;
    int button_count;
    int detail_count;
} menu_definition;

menu_definition *window_octavius_ui_build_menu_definition(int index);

void window_octavius_ui_build_menu_definition_init(void);

build_button *window_octavius_ui_build_menu_get_button_for(menu_definition *menu, building_type type);

#endif // WINDOW_OCTAVIUS_BUILD_MENU_MENU_DEFINITIONS_H
