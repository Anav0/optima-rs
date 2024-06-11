use std::ffi::CString;

use crate::{Color, DrawRectangle, DrawText, Font, MeasureText, MeasureTextEx};

use crate::colors::*;

pub(crate) unsafe fn draw_btn(font: &Font, text: *const i8, x: i32, y: i32, w: f32, h: f32, font_size: f32, color: Color) -> bool {
    DrawRectangle(x, y, w as i32, h as i32, color);

    let size = MeasureTextEx(*font, text, font_size, 1.0);

    let center_x = ((w - size.x) / 2.0) as i32;
    let center_y = ((h - size.y) / 2.0) as i32;
    DrawText(text, x + center_x, y + center_y, font_size as i32, BLACK);

    false
}
