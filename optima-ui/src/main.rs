#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod colors;

use colors::{BLACK, BLUE, RED, WHITE};
use std::ffi::CString;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    unsafe {
        let window_name = CString::new("Raylib - FFI").unwrap();

        InitWindow(1920, 1080, window_name.as_ptr());
        SetTargetFPS(60);

        while !WindowShouldClose() {
            BeginDrawing();

            ClearBackground(WHITE);

            let inital_x = 10;
            let inital_y = 10;
            let mut x = inital_x;
            let mut y = inital_y;
            let rows = 100;
            let w = 20;
            let h = 20;
            let gap = 5;

            for i in 1..2500 {
                let mut color = RED;
                if i % 2 == 0 {
                    color = BLACK;
                }
                DrawRectangle(x, y, w, h, color);
                x += w + gap;
                if i % rows == 0 {
                    y += h + gap;
                    x = inital_x;
                }
            }

            DrawCircle(50, 50, 50.0, RED);
            DrawCircle(100, 100, 50.0, BLUE);

            EndDrawing();
        }

        CloseWindow();
    }
}