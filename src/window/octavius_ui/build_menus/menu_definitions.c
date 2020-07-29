#include "menu_definitions.h"

#include "building/type.h"
#include "core/image_group.h"
#include "graphics/octavius_ui/build_button.h"
#include "window/octavius_ui/build_menu.h"


#define MAX(a, b) ( ((a) > (b)) ? (a) : (b) )

// param1: submenu
// param2: item index in submenu
// param3: calculated actual item index

// water related buildings

static build_button build_menu_water_buttons[] = {
    {   0, 0, 96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_WATER, 1, 1, 1 },
    {  96, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_WATER, 2, 1, 1 },
    { 160, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_WATER, 3, 1, 1 },
    { 224, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_WATER, 4, 1, 1 },
    0
};

static submenu_button_details build_menu_water_details[] = {
    { -12, 35, GROUP_BUILDING_RESERVOIR,  0, BUILDING_DRAGGABLE_RESERVOIR    },
    { -25, 17, GROUP_BUILDING_RESERVOIR,  1, BUILDING_DRAGGABLE_RESERVOIR, 1 },
    {   3, 55, GROUP_BUILDING_AQUEDUCT,   0, BUILDING_AQUEDUCT               },
    {   3, 55, GROUP_BUILDING_FOUNTAIN_1, 0, BUILDING_FOUNTAIN               },
    {   3, 55, GROUP_BUILDING_WELL,       0, BUILDING_WELL                   },
    0
};

// health related buildings

static build_button build_menu_health_buttons[] = {
    {   0, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_HEALTH, 1, 1, 1 },
    {  64, 0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_HEALTH, 2, 1, 1 },
    { 144, 0, 64, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_HEALTH, 3, 1, 1 },
    { 208, 0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_HEALTH, 4, 1, 1 },
    0
};

static submenu_button_details build_menu_health_details[] = {
    {   3, 55, GROUP_BUILDING_BARBER,          0, BUILDING_BARBER    },
    {   8, 33, GROUP_BUILDING_BATHHOUSE_WATER, 0, BUILDING_BATHHOUSE },
    {   3, 55, GROUP_BUILDING_DOCTOR,          0, BUILDING_DOCTOR    },
    { -20, 28, GROUP_BUILDING_HOSPITAL,        0, BUILDING_HOSPITAL  },
    0
};

// education buildings

static build_button build_menu_education_buttons[] = {
    {   0, 0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_EDUCATION, 1, 1, 1 },
    {  80, 0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_EDUCATION, 3, 1, 1 },
    { 160, 0, 96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_EDUCATION, 2, 1, 1 },
    0
};

static submenu_button_details build_menu_education_details[] = {
    {  15, 45, GROUP_BUILDING_SCHOOL,  0, BUILDING_SCHOOL  },
    {  15, 60, GROUP_BUILDING_LIBRARY, 0, BUILDING_LIBRARY },
    {  30, 30, GROUP_BUILDING_ACADEMY, 0, BUILDING_ACADEMY },
    0
};

// entertainment buildings

static build_button build_menu_entertainment_buttons[] = {
    {   0,   0,  80, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 8, 1, 1 },
    {  80,   0, 256, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 4, 1, 1 },
    {   0, 160, 112, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 7, 1, 1 },
    { 112, 160, 112, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 5, 1, 1 },
    { 224, 160, 112, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 6, 1, 1 },
    {   0, 320,  80, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 1, 1, 1 },
    {  80, 320,  96, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 2, 1, 1 },
    { 176, 320, 160, 160, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENTERTAINMENT, 3, 1, 1 },
    0
};

static submenu_button_details build_menu_entertainment_details[] = {
    {  15, 45, GROUP_BUILDING_CHARIOT_MAKER,    0, BUILDING_CHARIOT_MAKER    },
    {  70, 80, GROUP_BUILDING_HIPPODROME_1,     0, BUILDING_HIPPODROME       },
    {  30, 45, GROUP_BUILDING_ACTOR_COLONY,     0, BUILDING_ACTOR_COLONY     },
    {  30, 45, GROUP_BUILDING_GLADIATOR_SCHOOL, 0, BUILDING_GLADIATOR_SCHOOL },
    {  30, 45, GROUP_BUILDING_LION_HOUSE,       0, BUILDING_LION_HOUSE       },
    {  15, 60, GROUP_BUILDING_THEATER,          0, BUILDING_THEATER          },
    {  30, 45, GROUP_BUILDING_AMPHITHEATER,     0, BUILDING_AMPHITHEATER     },
    {  80, 45, GROUP_BUILDING_COLOSSEUM,        0, BUILDING_COLOSSEUM        },
    0
};

// government buildings

static build_button build_menu_government_buttons[] = {
    {   0,   0, 128, 260, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 2, 1, 1 },
    { 128, 130,  96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 1, 1, 1 },
    { 224, 130,  96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 6, 1, 1 },
    { 320, 130,  96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 7, 1, 1 },
    { 416, 130,  96, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 8, 1, 1 },
    { 128,   0, 128, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 3, 1, 1 },
    { 256,   0, 128, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 4, 1, 1 },
    { 384,   0, 128, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 5, 1, 1 },
    { 512,   0, 128, 260, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ADMINISTRATION, 9, 1, 1 },
    0
};

static submenu_button_details build_menu_government_details[] = {
    {  15,  55, GROUP_BUILDING_SENATE,           0, BUILDING_SENATE_UPGRADED    },
    {  20,  55, GROUP_BUILDING_FORUM,            0, BUILDING_FORUM              },
    {  20,  55, GROUP_BUILDING_STATUE,           0, BUILDING_SMALL_STATUE       },
    {  20,  30, GROUP_BUILDING_STATUE,           1, BUILDING_MEDIUM_STATUE      },
    {  20,  15, GROUP_BUILDING_STATUE,           2, BUILDING_LARGE_STATUE       },
    {  30,  30, GROUP_BUILDING_GOVERNORS_HOUSE,  0, BUILDING_GOVERNORS_HOUSE    },
    {   0,  15, GROUP_BUILDING_GOVERNORS_VILLA,  0, BUILDING_GOVERNORS_VILLA    },
    {  45,  15, GROUP_BUILDING_GOVERNORS_PALACE, 0, BUILDING_GOVERNORS_PALACE   },
    {  45, 110, GROUP_BUILDING_TRIUMPHAL_ARCH,   0, BUILDING_TRIUMPHAL_ARCH     },
    {  48,  58, GROUP_BUILDING_TRIUMPHAL_ARCH,   1, BUILDING_TRIUMPHAL_ARCH,  1 },
    0
};

// engineering buildings

static build_button build_menu_engineering_buttons[] = {
    {   0,   0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 4, 1, 1 },
    {  80,   0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 5, 1, 1 },
    { 160,   0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 6, 1, 1 },
    { 240,   0, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 8, 1, 1 },
    {   0, 130, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 1, 1, 1 },
    {  80, 130, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 3, 1, 1 },
    { 160, 130, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 2, 1, 1 },
    { 240, 130, 80, 130, IB_SUBMENU, 0, 0, window_octavius_build_button_menu_index, build_button_none, BUILD_MENU_ENGINEERING, 7, 1, 1 },
    0
};

static submenu_button_details build_menu_engineering_details[] = {
    {  45, 30, GROUP_BUILDING_BRIDGE,         3, BUILDING_LOW_BRIDGE,    1 },
    {  15, 30, GROUP_BUILDING_BRIDGE,         4, BUILDING_LOW_BRIDGE,    1 },
    { -15, 45, GROUP_BUILDING_BRIDGE,         5, BUILDING_LOW_BRIDGE,    1 },
    {  45, 15, GROUP_BUILDING_BRIDGE,         9, BUILDING_SHIP_BRIDGE,   1 },
    {  15, 20, GROUP_BUILDING_BRIDGE,        10, BUILDING_SHIP_BRIDGE,   1 },
    { -45, 35, GROUP_BUILDING_BRIDGE,        11, BUILDING_SHIP_BRIDGE,   1 },
    {  11, 35, GROUP_BUILDING_SHIPYARD,       1, BUILDING_SHIPYARD         },
    {  11, 35, GROUP_BUILDING_WHARF,          1, BUILDING_WHARF            },
    {  11, 55, GROUP_TERRAIN_GARDEN,          0, BUILDING_GARDENS          },
    {  11, 55, GROUP_BUILDING_ENGINEERS_POST, 0, BUILDING_ENGINEERS_POST   },
    {  11, 55, GROUP_TERRAIN_PLAZA,           0, BUILDING_PLAZA            },
    {  11, 35, GROUP_BUILDING_DOCK_2,         0, BUILDING_DOCK             },
    0,
};

// definitions

static menu_definition menu_definitions[] = {
    { build_menu_water_buttons,         build_menu_water_details,          0 },
    { build_menu_health_buttons,        build_menu_health_details,         0 },
    { build_menu_education_buttons,     build_menu_education_details,      0 },
    { build_menu_entertainment_buttons, build_menu_entertainment_details, 39 },
    { build_menu_government_buttons,    build_menu_government_details,    39 },
    { build_menu_engineering_buttons,   build_menu_engineering_details,   39 },
    0
};

int get_build_buttons_index(int submenu)
{
    switch (submenu) {
        case 3:  return  0; // water
        case 4:  return  1; // health
                            // religion
        case 6:  return  2; // education
        case 7:  return  3; // entertainment
        case 8:  return  4; // government
        case 9:  return  5; // engineering
                            // security
                            // industry
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

void window_octavius_ui_build_menu_definition_init(void)
{
    // calculate bounds of each menu and set width/height
    // set button_count for each menu
    int menu_idx = 0;
    while (menu_definitions[menu_idx].buttons > 0) {
        menu_definition *menu = &menu_definitions[menu_idx];
        int count = 0;

        int width = 0;
        int height = 0;

        while (menu->buttons[count].left_click_handler > 0) {
            build_button *btn = &menu->buttons[count];
            width = MAX(btn->x_offset + btn->width, width);
            height = MAX(btn->y_offset + btn->height, height);
            ++count;
        }

        menu->button_count = count;
        menu->width = width;
        menu->height = height;

        count = 0;
        while (menu->button_details[count].building_type > 0) {
            ++count;
        }
        menu->detail_count = count;

        ++menu_idx;
    }
}
