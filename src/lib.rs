mod utils;

use wasm_bindgen::prelude::*;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        #[cfg(feature = "debug_logging")]
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    const RED: Color = Color { r: 255, g: 0, b: 0 };
    const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };
    const PINK: Color = Color {
        r: 255,
        g: 20,
        b: 147,
    };
}
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Point {
    x: f32,
    y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Vec2 {
    dx: f32,
    dy: f32,
}

impl Vec2 {
    fn new(dx: f32, dy: f32) -> Vec2 {
        Vec2 { dx, dy }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2::new(self.dx * rhs, self.dy * rhs)
    }
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    fn l2_square(&self, other: &Point) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

impl std::ops::Add<Vec2> for Point {
    type Output = Point;
    fn add(self, rhs: Vec2) -> Point {
        Point::new(self.x + rhs.dx, self.y + rhs.dy)
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Vec2;
    fn sub(self, rhs: Point) -> Vec2 {
        Vec2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

struct Image {
    data: Vec<u8>,
    size: usize,
}

impl Image {
    fn new(size: usize) -> Image {
        let mut data = Vec::new();
        data.resize(size * size * 4, 255);
        Image { data, size }
    }

    fn set_coords(&mut self, x: usize, y: usize, color: Color) {
        let i = (y * self.size + x) * 4;
        self.data[i] = color.r;
        self.data[i + 1] = color.g;
        self.data[i + 2] = color.b;
    }
}

fn normal_pdf(mean: f32, sigma: f32, x: f32) -> f32 {
    1f32 / (sigma * (2f32 * std::f32::consts::PI).sqrt())
        * (-1f32 / 2f32 * ((x - mean) / sigma).powi(2)).exp()
}

fn select_candidate(p: Point, candidates: &[Point]) -> u8 {
    let mut closest_i = 100000000;
    let mut closest_dist = std::f32::MAX;
    for (i, c) in candidates.iter().enumerate() {
        let dist = p.l2_square(c);
        if dist < closest_dist {
            closest_dist = dist;
            closest_i = i;
        }
    }
    closest_i as u8
}

const CANDIDATE_COLORS: [Color; 5] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::YELLOW,
    Color::PINK,
];

#[wasm_bindgen]
pub fn render(size: usize, candidate_coords: Vec<f32>) -> Result<Vec<u8>, JsValue> {
    utils::set_panic_hook();

    let mut candidates = vec![];
    for i in (0..candidate_coords.len()).step_by(2) {
        candidates.push(Point::new(candidate_coords[i], candidate_coords[i + 1]));
    }
    let winners = election(size as i32, &candidates);
    let mut image = Image::new(size);
    for x in 0..size {
        for y in 0..size {
            let winner = winners[x * size + y];
            image.set_coords(x, y, CANDIDATE_COLORS[winner as usize]);
        }
    }

    Ok(image.data)
}

#[wasm_bindgen]
pub fn max_candidates() -> usize {
    CANDIDATE_COLORS.len()
}

pub fn election(size: i32, candidates: &[Point]) -> Vec<u8> {
    let sigma = 0.5f32;
    let num_sigma = 3.0;
    // The vote map is [0; 1], so we need to compute votes in [-num_sigma * sigma; 1 + num_sigma * sigma].
    let range = (size as f32 * sigma * num_sigma) as i32;
    let start = -range;
    let end = size + range;

    log!("Plotting from {} to {}", start, end);

    let padded_size = (end - start) as i32;
    let mut results = vec![0u8; padded_size.pow(2) as usize];

    // Compute voting results at each individual point.
    for x in start..end {
        for y in start..end {
            let at = Point::new(x as f32 / size as f32, y as f32 / size as f32);
            let winner = select_candidate(at, &candidates);

            let i = x - start;
            let j = y - start;
            results[(i * padded_size + j) as usize] = winner;
        }
    }

    // Neighbourhood weighting.
    let mut sample_locations = vec![];
    for x in -range..range {
        let p = normal_pdf(0f32, sigma, x as f32 / size as f32);
        sample_locations.push((x, p));
    }

    // Sum up all the votes for the neighbour of each point.
    let mut num_votes = vec![0f32; size.pow(2) as usize * candidates.len()];
    for x in 0..size {
        for y in start..end {
            // Sum up all the votes along the x-neighbourhood.
            let mut line_votes = vec![0f32; candidates.len()];
            for (dx, p) in sample_locations.iter() {
                let i = x + dx - start;
                let j = y - start;
                let winner = results[(i * padded_size + j) as usize];
                line_votes[winner as usize] += p;
            }

            // Add the summed votes to all points with the same x coordinate,
            // weighted by the distance along the y-axis.
            for (dy, p) in sample_locations.iter() {
                let yp = y + dy;
                if yp >= 0 && yp < size {
                    for i in 0..candidates.len() {
                        num_votes[((x * size) + yp) as usize * candidates.len() + i] +=
                            line_votes[i] * p;
                    }
                }
            }
        }
    }

    // Select the winner of the election for each point.
    let mut winners = vec![0u8; size.pow(2) as usize];
    for x in 0..size {
        for y in 0..size {
            let i = ((x * size) + y) as usize * candidates.len();
            let votes = &num_votes[i..i + candidates.len()];

            let winner = votes
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                .map(|(i, _)| i)
                .unwrap();
            winners[(x * size + y) as usize] = winner as u8;
        }
    }

    winners
}
