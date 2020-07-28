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
    int button_count;
    build_button *buttons;
    submenu_button_details *button_details;
    int offset_x;
    int width;
    int height;
} menu_definition;

menu_definition *window_octavius_ui_build_menu_definition(int index);

void window_octavius_ui_build_menu_definition_resize(void);

#endif // WINDOW_OCTAVIUS_BUILD_MENU_MENU_DEFINITIONS_H
