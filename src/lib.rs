mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// TODO(swj): Comment out for deployment, increases the binary size quite a lot.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

struct Map {
    data: Vec<u8>,
    height: usize,
}

impl Map {
    fn new(width: usize, height: usize) -> Map {
        let mut data = Vec::new();
        data.resize(width * height * 4, 255);
        Map { data, height }
    }

    fn set(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let i = (x * self.height + y) * 4;
        let (r, g, b) = color;
        self.data[i] = r;
        self.data[i + 1] = g;
        self.data[i + 2] = b;
    }
}

struct QuadTree {
    values: Vec<u8>,
}

impl QuadTree {
    fn new(size: usize) -> QuadTree {
        QuadTree {
            values: vec![0; size * size],
        }
    }
    fn insert(&mut self, x: usize, y: usize) {}
}

fn normal_pdf(mean: f32, sigma: f32, x: f32) -> f32 {
    1f32 / (sigma * (2f32 * std::f32::consts::PI).sqrt())
        * (-1f32 / 2f32 * ((x - mean) / sigma).powi(2)).exp()
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn l2_square(&self, other: &Point) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

fn select_candidate(p: Point, candidates: &[Point]) -> usize {
    let mut closest_i = 100000000;
    let mut closest_dist = std::f32::MAX;
    for (i, c) in candidates.iter().enumerate() {
        let dist = p.l2_square(c);
        if dist < closest_dist {
            closest_dist = dist;
            closest_i = i;
        }
    }
    closest_i
}

fn run_election(p: Point, candidates: &[Point], sample_locations: &[(f32, f32)]) -> usize {
    // log!("election at {:?} for candidates: {:?}", p, candidates);

    let mut num_votes = vec![0f32; candidates.len()];
    for (dx, px) in sample_locations {
        for (dy, py) in sample_locations {
            let at = Point {
                x: p.x + dx,
                y: p.y + dy,
            };
            let winner = select_candidate(at, candidates);
            num_votes[winner] += px * py;
        }
    }

    // log!("total votes: {:?}", num_votes);

    num_votes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap()
}

#[wasm_bindgen]
pub fn render(width: usize, height: usize) -> Result<Vec<u8>, JsValue> {
    let candidates = vec![
        Point { x: 0.5, y: 0.99 },
        Point { x: 0.07, y: 0.25 },
        Point { x: 0.93, y: 0.25 },
    ];
    let sigma = 0.5f32;
    let num_sigma = 3;
    let num_samples = 50;

    let bounds = (sigma * num_sigma as f32 * num_samples as f32) as i32;
    let mut sample_points = vec![];
    for x in -bounds..=bounds {
        let x = x as f32 / num_samples as f32;
        let p = normal_pdf(0f32, sigma, x);
        sample_points.push((x, p));
    }

    let colors = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];

    log!("sample points: {:?}", sample_points);

    let mut map = Map::new(width, height);

    let mut tree = QuadTree::new(256);
    for p in candidates.iter() {
        let winner = run_election(*p, &candidates, &sample_points);
        log!("at {:?} elected: {:}", p, winner);
        tree.insert(
            (p.x * width as f32) as usize,
            (p.y * height as f32) as usize,
        );
    }

    for x in 0..width {
        for y in 0..height {
            let winner = run_election(
                Point {
                    x: x as f32 / width as f32,
                    y: y as f32 / height as f32,
                },
                &candidates,
                &sample_points,
            );
            map.set(height - y - 1, x, colors[winner]);
        }
    }

    Ok(map.data)
}
