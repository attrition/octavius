#ifndef BUILDING_MENU_H
#define BUILDING_MENU_H

#include "building/type.h"

typedef enum {
    BUILD_MENU_VACANT_HOUSE = 0,
    BUILD_MENU_CLEAR_LAND = 1,
    BUILD_MENU_ROAD = 2,
    BUILD_MENU_WATER = 3,
    BUILD_MENU_HEALTH = 4,
    BUILD_MENU_TEMPLES = 5,
    BUILD_MENU_EDUCATION = 6,
    BUILD_MENU_ENTERTAINMENT = 7,
    BUILD_MENU_ADMINISTRATION = 8,
    BUILD_MENU_ENGINEERING = 9,
    BUILD_MENU_SECURITY = 10,
    BUILD_MENU_INDUSTRY = 11,
    BUILD_MENU_FARMS = 12,
    BUILD_MENU_RAW_MATERIALS = 13,
    BUILD_MENU_WORKSHOPS = 14,
    BUILD_MENU_SMALL_TEMPLES = 15,
    BUILD_MENU_LARGE_TEMPLES = 16,
    BUILD_MENU_FORTS = 17,
    BUILD_MENU_MAX = 18
} build_menu_group;

typedef enum {
    SIDEBAR_BUTTONS_VACANT_HOUSE = 0,
    SIDEBAR_BUTTONS_CLEAR_LAND = 8,
    SIDEBAR_BUTTONS_ROAD = 12,
    SIDEBAR_BUTTONS_WATER = 4,
    SIDEBAR_BUTTONS_HEALTH = 40,
    SIDEBAR_BUTTONS_TEMPLES = 28,
    SIDEBAR_BUTTONS_EDUCATION = 24,
    SIDEBAR_BUTTONS_ENTERTAINMENT = 20,
    SIDEBAR_BUTTONS_ADMINISTRATION = 16,
    SIDEBAR_BUTTONS_ENGINEERING = 44,
    SIDEBAR_BUTTONS_SECURITY = 36,
    SIDEBAR_BUTTONS_INDUSTRY = 32
} sidebar_buttons_group_index;

void building_menu_enable_all(void);

void building_menu_update(void);

int building_menu_count_items(int submenu);

int building_menu_check_index_enabled(int submenu, int index);

int building_menu_next_index(int submenu, int current_index);

building_type building_menu_type(int submenu, int item);

int building_menu_is_enabled(building_type type);

/**
 * Checks whether the building menu has changed.
 * Also marks the change as 'seen'.
 * @return True if the building menu has changed
 */
int building_menu_has_changed(void);

#endif // BUILDING_MENU_H
