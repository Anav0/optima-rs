#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use clap::Parser;
use clap::ValueEnum;
use image::io::Reader;
use optima_rust::swarm::max_value_of_range;
use optima_rust::swarm::min_value_of_range;
use std::fs;
use std::io::Cursor;
use std::ops::Bound;
use std::ops::RangeBounds;
use std::ops::RangeInclusive;
use std::path::Path;
use std::path::PathBuf;
use std::ptr;
use std::time::Duration;

use std::thread;
use std::{ffi::c_char, ffi::CString};

use optima_rust::{
    annealing::stop::NotGettingBetter,
    base::{Criterion, OptAlgorithm},
    swarm::{FnProblem, Particle, ParticleSwarm},
};

mod colors;
mod ui;

use colors::*;
use ui::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MathFnTwoArgs {
    Cormick,
    Booth,
    Simple,
}

#[derive(Parser)]
#[command(name = "Function selector")]
#[command(about = "Select function to optimize")]
struct Cli {
    #[arg(short, value_enum)]
    method: MathFnTwoArgs,

    #[arg(short, default_value_t = 0.0)]
    slowdown: f64,
}

const WIN_H: u32 = 1080;
const WIN_W: u32 = 1920;

const heightmap_h: u32 = 720;
const heightmap_w: u32 = 1080;

pub type MathFunction = dyn Fn(f64, f64) -> f64;

#[derive(Clone)]
struct FnBench<'a, R: RangeBounds<f64>> {
    pub name: String,
    pub x_range: R,
    pub y_range: R,
    pub global_minimum: (f64, f64, f64),
    pub func: &'a MathFunction,
}

//Some function to optimize.
//https://en.wikipedia.org/wiki/Test_functions_for_optimization
fn booth(x: f64, y: f64) -> f64 {
    f64::powf(x + 2.0 * y - 7.0, 2.0) + f64::powf(2.0 * x + y - 5.0, 2.0)
}

fn simple(x: f64, y: f64) -> f64 {
    x * y
}

fn cormick(x: f64, y: f64) -> f64 {
    (x + y).sin() + (x - y).powf(2.0) - 1.5 * x + 2.5 * y + 1.0
}

fn percent(value: f64, min: f64, max: f64) -> f64 {
    return (value - min) / (max - min);
}

unsafe fn draw_particle<R: RangeBounds<f64>>(p: &Particle, problem: &FnProblem<R>, color: Color) {
    let x_min = min_value_of_range(&problem.x_range).unwrap();
    let x_max = max_value_of_range(&problem.x_range).unwrap();

    let y_min = min_value_of_range(&problem.y_range).unwrap();
    let y_max = max_value_of_range(&problem.y_range).unwrap();

    let p_x: f64 = percent(p.x, x_min, x_max);
    let p_y: f64 = percent(p.y, y_min, y_max);

    let x: i32 = (f64::from(WIN_W) * p_x).round() as i32;
    let y: i32 = (f64::from(WIN_H) * p_y).round() as i32;

    DrawCircle(x, y, 5.0, color);
}

fn update_cstring_in_place(c_string: &mut CString, new_str: &str) {
    let new_len = new_str.len();
    let old_len = c_string.as_bytes().len();

    // Ensure the new string fits within the existing allocation
    if new_len > old_len {
        panic!(
            "New string is too long to fit in the existing CString buffer. {new_len} > {old_len}"
        );
    }

    let ptr: *mut c_char = c_string.as_ptr() as *mut c_char;
    unsafe {
        // Copy the new string bytes into the CString buffer
        ptr::copy_nonoverlapping(new_str.as_ptr(), ptr as *mut u8, new_len);
        // Null-terminate the CString
        *ptr.add(new_len) = 0;
    }
}

unsafe fn load_font<T: AsRef<Path>>(path: T) -> Option<Font> {
    let dir = std::env::current_dir().unwrap();
    let mut font_path = PathBuf::new();
    font_path.push(dir);
    font_path.push(path);

    let lato_font_path = CString::new(font_path.into_os_string().into_string().unwrap()).unwrap();
    let font = LoadFont(lato_font_path.as_ptr());

    let font_ptr: *const Font = &font;

    if !IsFontReady(font) || font_ptr.is_null() {
        return None;
    }

    Some(font)
}

fn generate_heightmap<R: RangeBounds<f64>>(
    file_name: &str,
    fn_to_optimize: &FnBench<R>,
    w: u32,
    h: u32,
) -> Vec<u8> {
    let mut pixels: Vec<u8> = Vec::with_capacity((w * h) as usize);

    let x_min = min_value_of_range(&fn_to_optimize.x_range).unwrap();
    let x_max = max_value_of_range(&fn_to_optimize.x_range).unwrap();

    let y_min = min_value_of_range(&fn_to_optimize.y_range).unwrap();
    let y_max = max_value_of_range(&fn_to_optimize.y_range).unwrap();

    let resolution_x = (x_max - x_min) / w as f64;
    let resolution_y = (y_max - y_min) / h as f64;

    let mut min_value = f64::MAX;
    let mut max_value = f64::MIN;
    for i in 0..w {
        for j in 0..h {
            let x = x_min + i as f64 * resolution_x;
            let y = y_min + j as f64 * resolution_y;
            let value = (fn_to_optimize.func)(x as f64, y as f64);
            min_value = f64::min(min_value, value);
            max_value = f64::max(max_value, value);
            pixels.push(value as u8);

            // println!("fn({x:3}, {y:3}) = {value:3}");
        }
    }

    for pixel in &mut pixels {
        let p = percent(*pixel as f64, min_value, max_value);
        assert!(p <= 1.0 && p >= 0.0);
        *pixel = (255.0 * p) as u8;
    }

    // fs::write(file_name, &pixels).expect("Failed to save heightmap!");
    image::save_buffer(file_name, &pixels, w, h, image::ExtendedColorType::L8)
        .expect("Failed to save heightmap!");

    pixels
}

fn main() {
    let cli = Cli::parse();

    let fn_to_optimize = match cli.method {
        MathFnTwoArgs::Booth => FnBench {
            global_minimum: (1.0, 3.0, 0.0),
            name: String::from("booth"),
            x_range: (-10.0..=10.0),
            y_range: (-10.0..=10.0),
            func: &booth,
        },
        MathFnTwoArgs::Simple => FnBench {
            global_minimum: (-10.0, 10.0, -100.0),
            name: String::from("simple"),
            x_range: (-10.0..=10.0),
            y_range: (-10.0..=10.0),
            func: &simple,
        },
        MathFnTwoArgs::Cormick => FnBench {
            global_minimum: (-0.54719, -1.54719, 0.0),
            name: String::from("cormick"),
            x_range: (-1.5..=4.0),
            y_range: (-3.0..=4.0),
            func: &cormick,
        },
    };

    let stop_criteria = NotGettingBetter::new(15000, 500, true);

    let mut swarm = ParticleSwarm::with_attraction(100, stop_criteria, 0.05, 0.04, 0.02);

    let problem = FnProblem::new(
        0,
        fn_to_optimize.x_range.clone(),
        fn_to_optimize.y_range.clone(),
    );

    let value_fn = |_problem: &FnProblem<RangeInclusive<f64>>, part: &Particle| {
        (fn_to_optimize.func)(part.x, part.y)
    };

    let mut criterion = Criterion::new(&|_, _| 0.0, &value_fn, true);

    unsafe {
        let window_name = CString::new("Swarm").unwrap();

        InitWindow(WIN_W as i32 as i32, WIN_H as i32, window_name.as_ptr());
        let monitor = GetCurrentMonitor();
        let refresh = GetMonitorRefreshRate(monitor);
        SetTargetFPS(refresh);

        let mut iter = 1;

        let mut iter_text = CString::new("Iter: 2000000").unwrap();
        let iter_text_pos = Vector2 { x: 50.0, y: 50.0 };
        let mut best_text = CString::new("Best: booth(1000, 1000) = 10000").unwrap();
        let best_text_pos = Vector2 { x: 50.0, y: 80.0 };
        let mut run_btn_text = CString::new("Run").unwrap();

        let font_size = 24.0;
        let font =
            load_font("optima-swarm-ui\\fonts\\Lato-Regular.ttf").expect("Failed to load font");

        // let pixels = generate_heightmap("heightmap.bmp", &fn_to_optimize, heightmap_w, heightmap_h);
        let pixels = generate_heightmap("heightmap.bmp", &fn_to_optimize, 500, 500);

        panic!("AA!");

        let known_optimum = Particle::new(
            fn_to_optimize.global_minimum.0,
            fn_to_optimize.global_minimum.1,
        );

        let draw_ui = &mut move |_problem: &FnProblem<RangeInclusive<f64>>,
                                 particles: &Vec<Particle>,
                                 best_index: usize| {
            BeginDrawing();

            ClearBackground(WHITE);

            draw_particle(&known_optimum, &problem, GOLD);

            let best = &particles[best_index];
            update_cstring_in_place(&mut iter_text, &format!("Iter: {}", iter));
            update_cstring_in_place(
                &mut best_text,
                &format!(
                    "Best: {}({:.3}, {:.3})",
                    fn_to_optimize.name,
                    best.x.round(),
                    best.y.round()
                ),
            );

            DrawTextEx(
                font,
                iter_text.as_ptr(),
                iter_text_pos,
                font_size,
                1.0,
                BLACK,
            );

            DrawTextEx(
                font,
                best_text.as_ptr(),
                best_text_pos,
                font_size,
                1.0,
                GOLD,
            );

            if draw_btn(
                &font,
                run_btn_text.as_ptr(),
                100,
                100,
                100.0,
                45.0,
                font_size,
                BTN_BG,
            ) {}

            let mut i = 0;
            for p in particles {
                if i == best_index {
                    continue;
                }
                draw_particle(p, &problem, BLUE);
                i += 1;
            }

            let p = &particles[best_index];
            draw_particle(p, &problem, RED);

            iter += 1;

            EndDrawing();

            WaitTime(cli.slowdown);

            WindowShouldClose()
        };

        swarm.register_insight(draw_ui);

        let particles = swarm.solve(problem, &mut criterion);
        let best = &particles[0];

        draw_particle(best, &problem, RED);

        while !WindowShouldClose() {}

        CloseWindow();
    }
}
