#include "menu_definitions.h"

#include "building/type.h"
#include "core/image_group.h"
#include "graphics/octavius_ui/build_button.h"
#include "window/octavius_ui/build_menu.h"

// param1: submenu
// param2: item index in submenu
// param3: calculated actual item index

// water related buildings

static build_button build_menu_water_buttons[] = {
    { 0,   0, 96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 3, 1, 1, 1 },
    { 96,  0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 3, 2, 1, 1 },
    { 160, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 3, 3, 1, 1 },
    { 224, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 3, 4, 1, 1 },
};

static submenu_button_details build_menu_water_definitions[] = {
    { -12, 35, GROUP_BUILDING_RESERVOIR,  BUILDING_RESERVOIR },
    {   3, 55, GROUP_BUILDING_AQUEDUCT,   BUILDING_AQUEDUCT  },
    {   3, 55, GROUP_BUILDING_FOUNTAIN_1, BUILDING_FOUNTAIN  },
    {   3, 55, GROUP_BUILDING_WELL,       BUILDING_WELL      },
};

// health related buildings

static build_button build_menu_health_buttons[] = {
    { 0,   0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 4, 1, 1, 1 },
    { 64,  0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 4, 2, 1, 1 },
    { 144, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 4, 3, 1, 1 },
    { 208, 0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 4, 4, 1, 1 },
};

static submenu_button_details build_menu_health_definitions[] = {
    {   3, 55, GROUP_BUILDING_BARBER,          BUILDING_BARBER    },
    {   8, 33, GROUP_BUILDING_BATHHOUSE_WATER, BUILDING_BATHHOUSE },
    {   3, 55, GROUP_BUILDING_DOCTOR,          BUILDING_DOCTOR    },
    { -20, 28, GROUP_BUILDING_HOSPITAL,        BUILDING_HOSPITAL  },
};

// education buildings

static build_button build_menu_education_buttons[] = {
    { 0,   0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 6, 1, 1, 1 },
    { 80,  0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 6, 3, 1, 1 },
    { 160, 0, 96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 6, 2, 1, 1 },
};

static submenu_button_details build_menu_education_definitions[] = {
    {   0, 45, GROUP_BUILDING_SCHOOL,  BUILDING_SCHOOL  },
    {  15, 60, GROUP_BUILDING_LIBRARY, BUILDING_LIBRARY },
    {  30, 30, GROUP_BUILDING_ACADEMY, BUILDING_ACADEMY },
};

static menu_definition menu_definitions[] = {
    { 4, build_menu_water_buttons,     build_menu_water_definitions,     0, 288, 130 },
    { 4, build_menu_health_buttons,    build_menu_health_definitions,    0, 288, 130 },
    { 3, build_menu_education_buttons, build_menu_education_definitions, 0, 256, 130 },
};

int get_build_buttons_index(int submenu)
{
    switch (submenu) {
        case 3:  return  0; // water
        case 4:  return  1; // health
        case 6:  return  2; // education
        default: return -1;
    }
}

menu_definition *window_octavius_ui_build_menu_definition(int submenu)
{
    int index = get_build_buttons_index(submenu);
    if (index != -1) {
        return &menu_definitions[index];
    }
    return 0;
}

