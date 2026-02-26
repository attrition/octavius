use std::ffi::c_int;

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Direction {
    Top = 0,
    TopRight = 1,
    Right = 2,
    BottomRight = 3,
    Bottom = 4,
    BottomLeft = 5,
    Left = 6,
    TopLeft = 7,
    None = 8,
    FigureReroute = 9,
    FigureLost = 10,
    FigureAttack = 11,
}

pub const DIR_FIGURE_AT_DESTINATION: i32 = 8;

#[unsafe(no_mangle)]
pub extern "C" fn calc_adjust_with_percentage(value: c_int, percentage: c_int) -> c_int {
    (percentage * value) / 100
}

#[unsafe(no_mangle)]
pub extern "C" fn calc_percentage(value: c_int, total: c_int) -> c_int {
    if total != 0 {
        (100 * value) / total
    } else {
        0
    }
}

/// Internal helper matching the static get_delta in C
#[inline]
fn get_delta(v1: c_int, v2: c_int) -> c_int {
    if v1 <= v2 {
        v2 - v1
    } else {
        v1 - v2
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calc_maximum_distance(x1: c_int, y1: c_int, x2: c_int, y2: c_int) -> c_int {
    let dx = get_delta(x1, x2);
    let dy = get_delta(y1, y2);
    if dx >= dy {
        dx
    } else {
        dy
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calc_distance_with_penalty(
    x1: c_int,
    y1: c_int,
    x2: c_int,
    y2: c_int,
    dist_to_entry1: c_int,
    dist_to_entry2: c_int,
) -> c_int {
    let mut penalty = if dist_to_entry1 > dist_to_entry2 {
        dist_to_entry1 - dist_to_entry2
    } else {
        dist_to_entry2 - dist_to_entry1
    };

    if dist_to_entry1 == -1 {
        penalty = 0;
    }

    penalty + calc_maximum_distance(x1, y1, x2, y2)
}

#[unsafe(no_mangle)]
pub extern "C" fn calc_general_direction(
    x_from: c_int,
    y_from: c_int,
    x_to: c_int,
    y_to: c_int,
) -> Direction {
    use std::cmp::Ordering::*;

    match (x_from.cmp(&x_to), y_from.cmp(&y_to)) {
        (Less, Greater) => Direction::TopRight,
        (Less, Equal) => Direction::Right,
        (Less, Less) => Direction::BottomRight,
        (Equal, Greater) => Direction::Top,
        (Equal, Less) => Direction::Bottom,
        (Greater, Greater) => Direction::TopLeft,
        (Greater, Equal) => Direction::Left,
        (Greater, Less) => Direction::BottomLeft,
        _ => Direction::None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calc_missile_shooter_direction(
    x_from: c_int,
    y_from: c_int,
    x_to: c_int,
    y_to: c_int,
) -> Direction {
    let dx = if x_from > x_to { x_from - x_to } else { x_to - x_from };
    let dy = if y_from > y_to { y_from - y_to } else { y_to - y_from };

    let percentage = match dx.cmp(&dy) {
        std::cmp::Ordering::Greater => calc_percentage(dx, dy),
        std::cmp::Ordering::Equal => 100,
        std::cmp::Ordering::Less => -calc_percentage(dy, dx),
    };

    if x_from == x_to {
        if y_from < y_to { Direction::Bottom } else { Direction::Top }
    } else if x_from > x_to {
        if y_from == y_to {
            Direction::Left
        } else if y_from > y_to {
            match percentage {
                p if p >= 400 => Direction::Left,
                p if p > -400 => Direction::TopLeft,
                _ => Direction::Top,
            }
        } else {
            match percentage {
                p if p >= 400 => Direction::Left,
                p if p > -400 => Direction::BottomLeft,
                _ => Direction::Bottom,
            }
        }
    } else { // x_from < x_to
        if y_from == y_to {
            Direction::Right
        } else if y_from > y_to {
            match percentage {
                p if p >= 400 => Direction::Right,
                p if p > -400 => Direction::TopRight,
                _ => Direction::Top,
            }
        } else {
            match percentage {
                p if p >= 400 => Direction::Right,
                p if p > -400 => Direction::BottomRight,
                _ => Direction::Bottom,
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calc_missile_direction(
    x_from: c_int,
    y_from: c_int,
    x_to: c_int,
    y_to: c_int,
) -> c_int {
    let dx = if x_from > x_to { x_from - x_to } else { x_to - x_from };
    let dy = if y_from > y_to { y_from - y_to } else { y_to - y_from };

    let percentage = match dx.cmp(&dy) {
        std::cmp::Ordering::Greater => calc_percentage(dx, dy),
        std::cmp::Ordering::Equal => 100,
        std::cmp::Ordering::Less => -calc_percentage(dy, dx),
    };

    if x_from == x_to {
        if y_from < y_to { 8 } else { 0 }
    } else if x_from > x_to {
        if y_from == y_to {
            12
        } else if y_from > y_to {
            match percentage {
                p if p >= 500 => 12,
                p if p >= 200 => 13,
                p if p > -200 => 14,
                p if p > -500 => 15,
                _ => 0,
            }
        } else {
            match percentage {
                p if p >= 500 => 12,
                p if p >= 200 => 11,
                p if p > -200 => 10,
                p if p > -500 => 9,
                _ => 8,
            }
        }
    } else { // x_from < x_to
        if y_from == y_to {
            4
        } else if y_from > y_to {
            match percentage {
                p if p >= 500 => 4,
                p if p >= 200 => 3,
                p if p > -200 => 2,
                p if p > -500 => 1,
                _ => 0,
            }
        } else {
            match percentage {
                p if p >= 500 => 4,
                p if p >= 200 => 5,
                p if p > -200 => 6,
                p if p > -500 => 7,
                _ => 8,
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn calc_bound(value: i32, min: i32, max: i32) -> i32 {
    value.clamp(min, max)
}
