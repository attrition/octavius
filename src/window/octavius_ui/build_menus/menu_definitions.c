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

static submenu_button_details build_menu_water_details[] = {
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

static submenu_button_details build_menu_health_details[] = {
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

static submenu_button_details build_menu_education_details[] = {
    {  15, 45, GROUP_BUILDING_SCHOOL,  BUILDING_SCHOOL  },
    {  15, 60, GROUP_BUILDING_LIBRARY, BUILDING_LIBRARY },
    {  30, 30, GROUP_BUILDING_ACADEMY, BUILDING_ACADEMY },
};

// entertainment buildings

static build_button build_menu_entertainment_buttons[] = {
    { 0,     0, 80,  160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 8, 1, 1 },
    { 80,    0, 256, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 4, 1, 1 },
    { 0,   160, 112, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 7, 1, 1 },
    { 112, 160, 112, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 5, 1, 1 },
    { 224, 160, 112, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 6, 1, 1 },
    { 0,   320, 80,  160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 1, 1, 1 },
    { 80,  320, 96,  160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 2, 1, 1 },
    { 176, 320, 160, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, 7, 3, 1, 1 },
};

static submenu_button_details build_menu_entertainment_details[] = {
    {  15, 45, GROUP_BUILDING_CHARIOT_MAKER,    BUILDING_CHARIOT_MAKER    },
    {  70, 80, GROUP_BUILDING_HIPPODROME_1,     BUILDING_HIPPODROME       },
    {  30, 45, GROUP_BUILDING_ACTOR_COLONY,     BUILDING_ACTOR_COLONY     },
    {  30, 45, GROUP_BUILDING_GLADIATOR_SCHOOL, BUILDING_GLADIATOR_SCHOOL },
    {  30, 45, GROUP_BUILDING_LION_HOUSE,       BUILDING_LION_HOUSE       },
    {  15, 60, GROUP_BUILDING_THEATER,          BUILDING_THEATER          },
    {  30, 45, GROUP_BUILDING_AMPHITHEATER,     BUILDING_AMPHITHEATER     },
    {  80, 45, GROUP_BUILDING_COLOSSEUM,        BUILDING_COLOSSEUM        },
};

// definitions

static menu_definition menu_definitions[] = {
    { 4, build_menu_water_buttons,         build_menu_water_details,         0, 288, 130 },
    { 4, build_menu_health_buttons,        build_menu_health_details,        0, 288, 130 },
    { 0, 0,                                0,                                0, 0,   0   },
    { 3, build_menu_education_buttons,     build_menu_education_details,     0, 256, 130 },
    { 8, build_menu_entertainment_buttons, build_menu_entertainment_details, 40, 336, 480 },
};

int get_build_buttons_index(int submenu)
{
    switch (submenu) {
        case 3:  return  0; // water
        case 4:  return  1; // health
        case 6:  return  3; // education
        case 7:  return  4; // entertainment
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

