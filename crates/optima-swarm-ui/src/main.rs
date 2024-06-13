#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use clap::Parser;
use clap::ValueEnum;
use image::io::Reader;
use image::ImageBuffer;
use image::Luma;
use optima_rust::swarm::max_value_of_range;
use optima_rust::swarm::min_value_of_range;
use std::ffi::CStr;
use std::fs;
use std::io;
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
    Bukin,
}

#[derive(Parser)]
#[command(name = "Optima")]
#[command(about = "Swarm algorithm")]
struct Cli {
    #[arg(short, help = "Function to optimize", value_enum)]
    method: MathFnTwoArgs,

    #[arg(short, help = "Wait time between simulation", default_value_t = 0.0)]
    slowdown: f64,

    #[arg(short, help = "Force heightmap recalculation", default_value_t = false)]
    force: bool,
}

const MATERIAL_MAP_DIFFUSE: usize = 0;
const WIN_H: u32 = 1080;
const WIN_W: u32 = 1920;

const HEIGHTMAP_W: u32 = 800;
const HEIGHTMAP_H: u32 = 800;
const HEIGHTMAP_POS: Vector3 = Vector3 {
    x: -8.0,
    y: 1.0,
    z: -8.0,
};
const HEIGHTMAP_SIZE: Vector3 = Vector3 {
    x: 16.0,
    y: 8.0,
    z: 16.0,
};

pub type MathFunction = dyn Fn(f64, f64) -> f64;

#[derive(Clone)]
struct FnBench<'a, R: RangeBounds<f64>> {
    pub name: String,
    pub x_range: R,
    pub y_range: R,
    pub v_min_found: f64,
    pub v_max_found: f64,
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

fn bukin(x: f64, y: f64) -> f64 {
    100.0 * f64::sqrt((y - 0.01 * x.powf(2.0)).abs() + 0.01 * (x + 10.0).abs())
}

fn percent(value: f64, min: f64, max: f64) -> f64 {
    return (value - min) / (max - min);
}

unsafe fn center_x(text: *const i8, font_size: i32, parent_x: u32, parent_w: u32) -> f32 {
    let text_w = MeasureText(text, font_size);
    parent_x as f32 + ((parent_w as f32 / 2.0) - (text_w as f32 / 2.0))
}

unsafe fn draw_particle<R: RangeBounds<f64>>(
    p: &Particle,
    problem: &FnProblem<R>,
    fn_to_optimize: &FnBench<R>,
    color: Color,
) {
    let v = (fn_to_optimize.func)(p.x, p.y);

    let x_min = min_value_of_range(&problem.x_range).unwrap();
    let x_max = max_value_of_range(&problem.x_range).unwrap();

    let y_min = min_value_of_range(&problem.y_range).unwrap();
    let y_max = max_value_of_range(&problem.y_range).unwrap();

    let v_min = fn_to_optimize.v_min_found;
    let v_max = fn_to_optimize.v_max_found;

    let p_x = percent(p.x, x_min, x_max) as f32;
    let p_y = percent(p.y, y_min, y_max) as f32;
    let p_v = percent(v, v_min, v_max) as f32;

    let x = HEIGHTMAP_POS.x + (HEIGHTMAP_SIZE.x * p_x); //HEIGHTMAP_POS.x + HEIGHTMAP_SIZE.x * p_x;
    let y = HEIGHTMAP_POS.y + (HEIGHTMAP_SIZE.y * p_v);
    let z = HEIGHTMAP_POS.z + (HEIGHTMAP_SIZE.z * p_y); // HEIGHTMAP_POS.z + HEIGHTMAP_SIZE.z * p_y;

    let size = 0.25;
    DrawCube(Vector3 { x, y, z }, size, size, size, color);
}

fn update_cstring_in_place(c_string: &mut CString, new_str: &str) {
    let new_len = new_str.len();
    let old_len = c_string.as_bytes().len();

    // Ensure the new string fits within the existing allocation
    if new_len > old_len {
        panic!(
            "New string is too long to fit in the existing CString buffer. {new_len} > {old_len}\n{new_str}"
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

fn take_samples<R: RangeBounds<f64>>(
    fn_to_optimize: &FnBench<R>,
    w: u32,
    h: u32,
) -> (Vec<f64>, f64, f64) {
    let mut samples: Vec<f64> = Vec::with_capacity((w * h) as usize);

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
            assert!(x <= x_max && x >= x_min);
            assert!(y <= y_max && y >= y_min);
            let value = (fn_to_optimize.func)(x as f64, y as f64);
            min_value = f64::min(min_value, value);
            max_value = f64::max(max_value, value);
            samples.push(value);
        }
    }

    (samples, min_value, max_value)
}

fn generate_heightmap(
    file_name: &str,
    mut values: Vec<f64>,
    v_max: f64,
    v_min: f64,
    w: u32,
    h: u32,
) -> io::Result<()> {
    let mut pixels: Vec<u8> = Vec::with_capacity(values.capacity());

    for value in &mut values {
        let p = percent(*value as f64, v_min, v_max);
        pixels.push((255.0 * p) as u8);
    }

    let buffer: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_vec(w, h, pixels).unwrap();

    buffer
        .save_with_format(file_name, image::ImageFormat::Png)
        .expect("Failed to save buffer as PNG!");

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let mut fn_to_optimize = match cli.method {
        MathFnTwoArgs::Bukin => FnBench {
            global_minimum: (-10.0, 1.0, 0.0),
            name: String::from("bukin"),
            x_range: (-15.0..=-5.0),
            y_range: (-3.0..=3.0),
            func: &bukin,
            v_max_found: f64::MIN,
            v_min_found: f64::MAX,
        },
        MathFnTwoArgs::Booth => FnBench {
            global_minimum: (1.0, 3.0, 0.0),
            name: String::from("booth"),
            x_range: (-10.0..=10.0),
            y_range: (-10.0..=10.0),
            func: &booth,
            v_max_found: f64::MIN,
            v_min_found: f64::MAX,
        },
        MathFnTwoArgs::Simple => FnBench {
            global_minimum: (-10.0, 10.0, -100.0),
            name: String::from("simple"),
            x_range: (-10.0..=10.0),
            y_range: (-10.0..=10.0),
            func: &simple,
            v_max_found: f64::MIN,
            v_min_found: f64::MAX,
        },
        MathFnTwoArgs::Cormick => FnBench {
            global_minimum: (-0.54719, -1.54719, 0.0),
            name: String::from("cormick"),
            x_range: (-1.5..=4.0),
            y_range: (-3.0..=4.0),
            func: &cormick,
            v_max_found: f64::MIN,
            v_min_found: f64::MAX,
        },
    };

    let stop_criteria = NotGettingBetter::new(15000, 100, true);

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

        let known_optimum = Particle::new(
            fn_to_optimize.global_minimum.0,
            fn_to_optimize.global_minimum.1,
        );

        let font_size = 24.0;
        let b_font_size = 36.0;

        let spacing = 35.0;

        let info_texts_pos = Vector2 { x: 50.0, y: 50.0 };
        let mut info_texts_pos_ex = info_texts_pos;

        let mut iter_text = CString::new("Iteration: 2000000").unwrap();
        let mut best_text =
            CString::new("Best found: some_fn_name_even_long_one_should_fit(1000, 1000) = 10000")
                .unwrap();
        let run_btn_text = CString::new("Run").unwrap();
        let mut finished_text = CString::new("Done!").unwrap();
        let finished_text_pos_x = center_x(finished_text.as_ptr(), b_font_size as i32, 0, WIN_W);
        let finished_text_pos = Vector2 {
            x: finished_text_pos_x,
            y: 80.0,
        };

        let mut known_optimum_text = CString::new(format!(
            "Known minimum: {}({:.3}, {:.3}) = {:.3}",
            fn_to_optimize.name,
            fn_to_optimize.global_minimum.0,
            fn_to_optimize.global_minimum.1,
            fn_to_optimize.global_minimum.2
        ))
        .unwrap();

        let font = load_font("crates\\optima-swarm-ui\\fonts\\Lato-Regular.ttf")
            .expect("Failed to load font");

        let heightmap_filename = format!(
            "{}_{}_{}.png",
            fn_to_optimize.name, HEIGHTMAP_W, HEIGHTMAP_H
        );

        let (samples, v_min, v_max) = take_samples(&fn_to_optimize, HEIGHTMAP_W, HEIGHTMAP_H);
        fn_to_optimize.v_max_found = v_max;
        fn_to_optimize.v_min_found = v_min;

        if !Path::new(&heightmap_filename).exists() || cli.force {
            generate_heightmap(
                &heightmap_filename,
                samples,
                v_max,
                v_min,
                HEIGHTMAP_W,
                HEIGHTMAP_H,
            )
            .expect(&format!(
                "Failed to generate heightmap for: {heightmap_filename}"
            ));
        }

        let mut camera: Camera = Camera {
            position: Vector3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            up: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            fovy: 45.0,
            projection: CameraProjection_CAMERA_PERSPECTIVE,
            target: Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };

        let image = LoadImage(CString::new(heightmap_filename).unwrap().as_ptr());

        let texture = LoadTextureFromImage(image);

        let heightmap_mesh = GenMeshHeightmap(image, HEIGHTMAP_SIZE);

        let heightmap_model = LoadModelFromMesh(heightmap_mesh);
        let mut materials = std::slice::from_raw_parts(
            heightmap_model.materials,
            heightmap_model.materialCount as usize,
        );
        let mut mappa = std::slice::from_raw_parts_mut(materials[0].maps, 4);

        mappa[MATERIAL_MAP_DIFFUSE].texture = texture;

        UnloadImage(image);

        let func = fn_to_optimize.clone();

        let mut iter = 0;
        let draw_ui = &mut move |problem: &FnProblem<RangeInclusive<f64>>,
                                 particles: &Vec<Particle>,
                                 best_index: usize,
                                 finished: bool| {
            UpdateCamera(&mut camera, CameraMode_CAMERA_ORBITAL);

            BeginDrawing();

            ClearBackground(WHITE);

            BeginMode3D(camera);

            DrawModel(heightmap_model, HEIGHTMAP_POS, 1.0, GREEN);
            DrawGrid(20, 1.0);

            draw_particle(&known_optimum, &problem, &func, GOLD);

            let mut i = 0;
            for p in particles {
                if i == best_index {
                    continue;
                }
                draw_particle(p, &problem, &func, BLUE);
                i += 1;
            }

            draw_particle(&particles[best_index], &problem, &func, RED);

            if !finished {
                iter += 1;
            }

            EndMode3D();

            let best = &particles[best_index];
            if !finished {
                update_cstring_in_place(&mut iter_text, &format!("Iteration: {}", iter));
                update_cstring_in_place(
                    &mut best_text,
                    &format!(
                        "Best found: {}({:.3}, {:.3}) = {:.3}",
                        fn_to_optimize.name,
                        best.x,
                        best.y,
                        (func.func)(best.x, best.y),
                    ),
                );
            }

            if finished {
                DrawTextEx(
                    font,
                    finished_text.as_ptr(),
                    finished_text_pos,
                    b_font_size,
                    1.0,
                    BLACK,
                );
            }

            DrawTextEx(
                font,
                iter_text.as_ptr(),
                info_texts_pos_ex,
                font_size,
                1.0,
                BLACK,
            );

            info_texts_pos_ex.y += spacing;

            DrawTextEx(
                font,
                best_text.as_ptr(),
                info_texts_pos_ex,
                font_size,
                1.0,
                GOLD,
            );

            info_texts_pos_ex.y += spacing;

            DrawTextEx(
                font,
                known_optimum_text.as_ptr(),
                info_texts_pos_ex,
                font_size,
                1.0,
                GOLD,
            );

            info_texts_pos_ex = info_texts_pos;

            EndDrawing();

            WaitTime(cli.slowdown);

            WindowShouldClose()
        };

        swarm.register_insight(draw_ui);

        let particles = swarm.solve(problem.clone(), &mut criterion);

        while !WindowShouldClose() {
            draw_ui(&problem, &particles, 0, true);
        }

        UnloadTexture(texture);
        UnloadModel(heightmap_model);

        CloseWindow();
    }
}
