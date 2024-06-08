#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::thread;
use std::ptr;

use optima_rust::{
    annealing::stop::{MaxSteps, NotGettingBetter},
    base::{Criterion, OptAlgorithm, Solution},
    swarm::{FnProblem, Particle, ParticleSwarm},
};

mod colors;

use colors::*;
use std::{ffi::CString, ffi::c_char, time::Duration};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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

fn draw_ui(problem: &FnProblem, particles: &Vec<Particle>) {}

fn percent(value: f64, min: f64, max: f64) -> f64 {
    return (value - min) / (max - min);
}

unsafe fn draw_particle(p: &Particle, min: f64, max: f64, color: Color) {
        let p_x: f64 = percent(p.x, min, max);
        let p_y: f64 = percent(p.y, min, max);

        let x: i32 = (f64::from(WIN_W) * p_x).round() as i32;
        let y: i32 = (f64::from(WIN_H) * p_y).round() as i32;

        DrawCircle(x, y, 10.0, color);
}
    fn update_cstring_in_place(c_string: &mut CString, new_str: &str) {
        let new_len = new_str.len();
        let old_len = c_string.as_bytes().len();
        
        // Ensure the new string fits within the existing allocation
        if new_len > old_len {
            panic!("New string is too long to fit in the existing CString buffer. {new_len} > {old_len}");
        }

        let ptr: *mut c_char = c_string.as_ptr() as *mut c_char;
        unsafe {
            // Copy the new string bytes into the CString buffer
            ptr::copy_nonoverlapping(new_str.as_ptr(), ptr as *mut u8, new_len);
            // Null-terminate the CString
            *ptr.add(new_len) = 0;
        }
    }

fn main() {
    let booth_bench = FnBench {
        global_minimum: (1.0, 3.0, 0.0),
        max: 10.0,
        min: -10.0,
        name: String::from("booth"),
        func: &booth,
    };

    let stop_criteria = NotGettingBetter::new(15000, 500, true);

    let mut swarm = ParticleSwarm::new(100, stop_criteria);

    let problem = FnProblem::new(0, booth_bench.max, booth_bench.min);

    let value_fn = |_problem: &FnProblem, part: &Particle| (booth_bench.func)(part.x, part.y);

    let mut criterion = Criterion::new(&|_, _| 0.0, &value_fn, true);

    unsafe {
        let window_name = CString::new("Swarm").unwrap();

        InitWindow(WIN_W, WIN_H, window_name.as_ptr());
        let monitor = GetCurrentMonitor();
        let refresh = GetMonitorRefreshRate(monitor);
        SetTargetFPS(refresh);

        let max = booth_bench.max;
        let min = booth_bench.min;

        let mut iter = 1;

        let mut iter_text = CString::new("Iter: 2000000").unwrap();
        let iter_text_pos = Vector2 {x :50.0, y: 50.0 };
        let mut best_text = CString::new("Best: booth(1000, 1000) = 10000").unwrap();
        let best_text_pos = Vector2 {x :50.0, y: 80.0 };

        let lato_font_path = CString::new("D:/Projects/optima-rust/optima-ui/fonts/Lato-Regular.ttf").unwrap();
        let font = LoadFont(lato_font_path.as_ptr());

        let max_try = 10;
        let mut tried = 0;
        while !IsFontReady(font) {
            if tried > max_try { panic!("Failed to load font!"); };
            thread::sleep(Duration::from_millis(500));
            tried+=1;
        }

        let font_size = 24.0;

        let draw_ui = &mut move |_problem: &FnProblem, particles: &Vec<Particle>, best_index: usize| {
            BeginDrawing();

            ClearBackground(WHITE);

            let best = &particles[best_index];
            update_cstring_in_place(&mut iter_text, &format!("Iter: {}", iter));
            update_cstring_in_place(&mut best_text, &format!("Best: booth({:.3}, {:.3})", best.x.round(), best.y.round()));

            DrawTextEx(font, iter_text.as_ptr(), iter_text_pos, font_size, 1.0, BLACK);
            DrawTextEx(font, best_text.as_ptr(), best_text_pos, font_size, 1.0, GOLD);

            let mut i = 0;
            for p in particles {
                if i == best_index { continue; }
                draw_particle(p, min, max, BLUE);
                i += 1;
            }

            let p = &particles[best_index];
            draw_particle(p, min, max, RED);

            iter += 1;

            EndDrawing();

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
