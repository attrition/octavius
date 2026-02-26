use std::ffi::c_int;
use crate::time::{time_get_millis, TimeMillis};

#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SpeedDirection {
    Negative = -1,
    Stopped = 0,
    Positive = 1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SpeedType {
    pub start_time: TimeMillis,
    pub total_time: TimeMillis,
    pub last_speed_check: TimeMillis,
    pub speed_difference: f64,
    pub desired_speed: f64,
    pub current_speed: f64,
    pub adjusted_current_speed: f64,
    pub cumulative_delta: f64,
    pub fine_position: f64,
    pub adjust_for_time: c_int,
}

const FRAME_TIME: f64 = 16.67;
pub const SPEED_CHANGE_IMMEDIATE: TimeMillis = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn speed_clear(speed: *mut SpeedType) {
    if speed.is_null() { return; }
    let s = unsafe { &mut *speed };
    s.cumulative_delta = 0.0;
    s.fine_position = 0.0;
    s.desired_speed = 0.0;
    s.current_speed = 0.0;
    s.speed_difference = 0.0;
    s.start_time = 0;
    s.total_time = 0;
    s.last_speed_check = unsafe { time_get_millis() };
}

#[inline]
fn adjust_speed_for_elapsed_time(delta: f64, adjust_for_time: bool, last_time: TimeMillis) -> f64 {
    if adjust_for_time {
        (delta / FRAME_TIME) * (unsafe { time_get_millis() } - last_time) as f64
    } else {
        delta
    }
}

#[inline]
fn adjust_speed_for_frame_time(delta: f64, adjust_for_time: bool, last_time: TimeMillis) -> f64 {
    if adjust_for_time {
        let diff = (unsafe { time_get_millis() } - last_time) as f64;
        if diff != 0.0 {
            (delta / diff) * FRAME_TIME
        } else {
            delta
        }
    } else {
        delta
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn speed_set_target(
    speed: *mut SpeedType,
    new_speed: f64,
    total_time: TimeMillis,
    adjust_for_time: c_int,
) {
    if speed.is_null() { return; }
    let s = unsafe { &mut *speed };
    s.adjust_for_time = adjust_for_time;
    if new_speed == s.desired_speed {
        return;
    }

    if total_time == SPEED_CHANGE_IMMEDIATE {
        s.desired_speed = new_speed;
        s.current_speed = new_speed;
        s.total_time = total_time;
        if adjust_for_time == 0 && unsafe { time_get_millis() } != s.last_speed_check {
            s.adjusted_current_speed = adjust_speed_for_frame_time(new_speed, true, s.last_speed_check);
        } else {
            s.adjusted_current_speed = new_speed;
        }
        return;
    }

    s.cumulative_delta = 0.0;
    s.fine_position = 0.0;
    let base_speed = if adjust_for_time != 0 { s.adjusted_current_speed } else { s.current_speed };
    s.speed_difference = base_speed - new_speed;
    s.desired_speed = new_speed;
    s.start_time = unsafe { time_get_millis() };
    s.total_time = total_time;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn speed_invert(speed: *mut SpeedType) {
    if speed.is_null() { return; }
    let s = unsafe { &*speed };
    unsafe {
        speed_set_target(speed, -s.current_speed, SPEED_CHANGE_IMMEDIATE, s.adjust_for_time);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn speed_get_current_direction(speed: *const SpeedType) -> SpeedDirection {
    if speed.is_null() { return SpeedDirection::Stopped; }
    let s = unsafe { &*speed };
    if s.current_speed == 0.0 {
        SpeedDirection::Stopped
    } else if s.current_speed > 0.0 {
        SpeedDirection::Positive
    } else {
        SpeedDirection::Negative
    }
}

#[inline]
fn handle_fine_position(speed: &mut SpeedType, delta: f64) -> c_int {
    let delta_rounded = delta as i32;
    speed.fine_position += delta - delta_rounded as f64;
    let extra_position = speed.fine_position as i32;
    speed.fine_position -= extra_position as f64;
    (delta_rounded + extra_position) as c_int
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn speed_get_delta(speed: *mut SpeedType) -> c_int {
    if speed.is_null() { return 0; }
    let now = unsafe { time_get_millis() };
    let s = unsafe { &mut *speed };

    if s.adjust_for_time != 0 && s.last_speed_check == now {
        return 0;
    }

    let delta: f64;
    let elapsed = now - s.start_time;
    let adjust_for_time_bool = s.adjust_for_time != 0;
    let desired = adjust_speed_for_elapsed_time(s.desired_speed, adjust_for_time_bool, s.last_speed_check);

    if s.total_time == SPEED_CHANGE_IMMEDIATE {
        delta = desired;
    } else if s.current_speed == s.desired_speed || elapsed > s.total_time * 4 {
        delta = desired;
        s.current_speed = s.desired_speed;
        s.adjusted_current_speed = s.desired_speed;
    } else {
        if elapsed == 0 {
            delta = adjust_speed_for_elapsed_time(s.current_speed, adjust_for_time_bool, s.last_speed_check);
        } else {
            let full_delta = s.speed_difference * (s.total_time as f64 / FRAME_TIME);
            let exponent = (-(elapsed as f64) / s.total_time as f64).exp();
            let mut d = full_delta - full_delta * exponent - s.cumulative_delta;
            s.cumulative_delta += d;
            d += desired;
            delta = d;
            s.current_speed = adjust_speed_for_frame_time(delta, adjust_for_time_bool, s.last_speed_check);
            s.adjusted_current_speed = s.current_speed;
        }
    }

    s.last_speed_check = now;
    handle_fine_position(s, delta)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn speed_is_changing(speed: *const SpeedType) -> c_int {
    if speed.is_null() { return 0; }
    let s = unsafe { &*speed };
    (s.current_speed != s.desired_speed) as c_int
}
