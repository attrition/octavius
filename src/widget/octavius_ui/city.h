#ifndef WIDGET_OCTAVIUS_UI_CITY_H
#define WIDGET_OCTAVIUS_UI_CITY_H

#include "graphics/tooltip.h"
#include "input/mouse.h"

void widget_octavius_ui_city_draw_background(void);

void widget_octavius_ui_city_draw_foreground(void);
void widget_octavius_ui_city_draw_foreground_military(void);

int widget_octavius_ui_city_handle_mouse(const mouse *m);
int widget_octavius_ui_city_handle_mouse_build_menu(const mouse *m);

int widget_octavius_ui_city_get_tooltip_text(tooltip_context *c);

void widget_octavius_ui_city_init(void);

#endif // WIDGET_OCTAVIUS_UI_CITY_H
