#include "city.h"

#include "graphics/screen.h"
#include "input/scroll.h"
#include "map/grid.h"
#include "widget/minimap.h"

//static struct {
//
//} data;

void widget_octavius_ui_city_draw_background(void)
{

}

void widget_octavius_ui_city_draw_foreground(void)
{
    int map_height = map_grid_height() * 2;

    if (scroll_in_progress()) {
        widget_minimap_invalidate();
    }

    widget_minimap_draw(0, screen_height() - map_height, map_grid_width(), map_height, 1);
}

void widget_octavius_ui_city_draw_foreground_military(void)
{

}

int widget_octavius_ui_city_handle_mouse(const mouse *m)
{
    return 0;
}

int widget_octavius_ui_city_handle_mouse_build_menu(const mouse *m)
{
    return 0;
}

int widget_octavius_ui_city_get_tooltip_text(void)
{
    return 0;
}
