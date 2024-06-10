#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use clap::Parser;
use clap::ValueEnum;
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

use colors::*;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MathFnTwoArgs {
    Cormack,
    Booth,
}

#[derive(Parser)]
#[command(name = "Function selector")]
#[command(about = "Select function to optimize")]
struct Cli {
    #[arg(short, value_enum)]
    method: MathFnTwoArgs,

    #[arg(short, default_value_t = 0)]
    slowdown: u64,
}

const WIN_H: i32 = 1080;
const WIN_W: i32 = 1920;

pub type MathFunction = dyn Fn(f64, f64) -> f64;

struct FnBench<'a> {
    pub name: String,
    pub min: f64,
    pub max: f64,
    pub global_minimum: (f64, f64, f64),
    pub func: &'a MathFunction,
}

//Some function to optimize.
//https://en.wikipedia.org/wiki/Test_functions_for_optimization
fn booth(x: f64, y: f64) -> f64 {
    f64::powf(x + 2.0 * y - 7.0, 2.0) + f64::powf(2.0 * x + y - 5.0, 2.0)
}

fn cormick(x: f64, y: f64) -> f64 {
    (x + y).sin() + (x - y).powf(2.0) - 1.5 * x + 2.5 * y + 1.0
}

fn percent(value: f64, min: f64, max: f64) -> f64 {
    return (value - min) / (max - min);
}

unsafe fn draw_particle(p: &Particle, min: f64, max: f64, color: Color) {
    let p_x: f64 = percent(p.x, min, max);
    let p_y: f64 = percent(p.y, min, max);

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

fn main() {
    let cli = Cli::parse();

    let fn_to_optimize = match cli.method {
        MathFnTwoArgs::Booth => FnBench {
            global_minimum: (1.0, 3.0, 0.0),
            max: 10.0,
            min: -10.0,
            name: String::from("booth"),
            func: &booth,
        },
        MathFnTwoArgs::Cormack => FnBench {
            global_minimum: (-0.54719, -1.54719, 0.0),
            max: 10.0,
            min: -10.0,
            name: String::from("cormack"),
            func: &booth,
        },
    };

    let stop_criteria = NotGettingBetter::new(15000, 500, true);

    let mut swarm = ParticleSwarm::new(100, stop_criteria);

    let problem = FnProblem::new(0, fn_to_optimize.max, fn_to_optimize.min);

    let value_fn = |_problem: &FnProblem, part: &Particle| (fn_to_optimize.func)(part.x, part.y);

    let mut criterion = Criterion::new(&|_, _| 0.0, &value_fn, true);

    unsafe {
        let window_name = CString::new("Swarm").unwrap();

        InitWindow(WIN_W, WIN_H, window_name.as_ptr());
        let monitor = GetCurrentMonitor();
        let refresh = GetMonitorRefreshRate(monitor);
        SetTargetFPS(refresh);

        let max = fn_to_optimize.max;
        let min = fn_to_optimize.min;

        let mut iter = 1;

        let mut iter_text = CString::new("Iter: 2000000").unwrap();
        let iter_text_pos = Vector2 { x: 50.0, y: 50.0 };
        let mut best_text = CString::new("Best: booth(1000, 1000) = 10000").unwrap();
        let best_text_pos = Vector2 { x: 50.0, y: 80.0 };

        let font_size = 24.0;
        let font =
            load_font("optima-swarm-ui\\fonts\\Lato-Regular.ttf").expect("Failed to load font");

        let sleep_dur = Duration::from_millis(cli.slowdown);
        let draw_ui =
            &mut move |_problem: &FnProblem, particles: &Vec<Particle>, best_index: usize| {
                BeginDrawing();

                ClearBackground(WHITE);

                let best = &particles[best_index];
                update_cstring_in_place(&mut iter_text, &format!("Iter: {}", iter));
                update_cstring_in_place(
                    &mut best_text,
                    &format!("Best: {}({:.3}, {:.3})", fn_to_optimize.name, best.x.round(), best.y.round()),
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

                let mut i = 0;
                for p in particles {
                    if i == best_index {
                        continue;
                    }
                    draw_particle(p, min, max, BLUE);
                    i += 1;
                }

                let p = &particles[best_index];
                draw_particle(p, min, max, RED);

                iter += 1;

                EndDrawing();

                thread::sleep(sleep_dur);

                WindowShouldClose()
            };

        swarm.register_insight(draw_ui);

        let particles = swarm.solve(problem, &mut criterion);
        let best = &particles[0];

        draw_particle(best, min, max, GOLD);

        while !WindowShouldClose() {}

        CloseWindow();
    }
}
