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
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
struct Vec2 {
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
    fn new(x: f32, y: f32) -> Point {
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
    height: usize,
    width: usize,
}

impl Image {
    fn new(width: usize, height: usize) -> Image {
        let mut data = Vec::new();
        data.resize(width * height * 4, 255);
        Image {
            data,
            height,
            width,
        }
    }

    fn set(&mut self, p: Point, color: Color) {
        let x = (p.x * (self.width - 1) as f32).round() as usize;
        let y = (p.y * (self.height - 1) as f32).round() as usize;
        self.set_coords(x, y, color);
    }

    fn set_coords(&mut self, x: usize, y: usize, color: Color) {
        let i = (x * self.height + y) * 4;
        self.data[i] = color.r;
        self.data[i + 1] = color.g;
        self.data[i + 2] = color.b;
    }

    fn line(&mut self, from: Point, to: Point, color: Color) {
        let dx = if from.x == to.x { 0.0 } else { 1.0 };
        let dy = if from.y == to.y { 0.0 } else { 1.0 };
        let dp = Vec2::new(dx / self.height as f32, dy / self.width as f32);
        let mut cur = from;
        while cur <= to {
            self.set(cur, color);
            cur = cur + dp;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TreeState {
    Inner(Box<[QuadTree; 4]>),
    Leaf(Option<Point>),
}

#[derive(Clone, Debug, PartialEq)]
struct QuadTree {
    from: Point,
    to: Point,
    tree: TreeState,
}

impl QuadTree {
    fn new(from: Point, to: Point) -> QuadTree {
        QuadTree {
            from,
            to,
            tree: TreeState::Leaf(None),
        }
    }

    fn default() -> QuadTree {
        QuadTree::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0))
    }

    fn insert(&mut self, p: Point) {
        let maybe_old_p: Option<Point>;

        match &mut self.tree {
            TreeState::Inner(children) => {
                for child in children.iter_mut() {
                    if p.x >= child.from.x
                        && p.x <= child.to.x
                        && p.y >= child.from.y
                        && p.y <= child.to.y
                    {
                        child.insert(p);
                        return;
                    }
                }
                unreachable!("Failed to insert {:?} into {:?}", p, self);
            }
            TreeState::Leaf(Some(old_p)) => {
                if p == *old_p {
                    return;
                }
                maybe_old_p = Some(*old_p);
            }
            TreeState::Leaf(None) => {
                let mut leaf = TreeState::Leaf(Some(p));
                std::mem::swap(&mut leaf, &mut self.tree);
                return;
            }
        }

        // Workaround for borrow checker, really this should be inside the Leaf(Some()) branch.
        self.split();
        self.insert(maybe_old_p.unwrap());
        self.insert(p);
    }

    fn visit_leaves<F>(&self, f: &mut F)
    where
        F: FnMut(Point, Point),
    {
        match &self.tree {
            TreeState::Inner(children) => children.iter().for_each(|c| c.visit_leaves(f)),
            TreeState::Leaf(_) => f(self.from, self.to),
        }
    }

    fn draw(&self, image: &mut Image) {
        // Two-step drawing process to make sure outlines don't overwrite points for the values.
        self.draw_outlines(image);
        self.draw_values(image);
    }

    fn draw_outlines(&self, image: &mut Image) {
        match &self.tree {
            TreeState::Inner(children) => children.iter().for_each(|c| c.draw_outlines(image)),
            TreeState::Leaf(_) => {
                image.line(self.from, Point::new(self.from.x, self.to.y), Color::BLACK);
                image.line(
                    Point::new(self.to.x, self.from.y),
                    Point::new(self.to.x, self.to.y),
                    Color::BLACK,
                );
                image.line(self.from, Point::new(self.to.x, self.from.y), Color::BLACK);
                image.line(
                    Point::new(self.from.x, self.to.y),
                    Point::new(self.to.x, self.to.y),
                    Color::BLACK,
                );
            }
        }
    }

    fn draw_values(&self, image: &mut Image) {
        match &self.tree {
            TreeState::Inner(children) => children.iter().for_each(|c| c.draw_values(image)),
            TreeState::Leaf(value) => {
                if let Some(p) = value {
                    image.set(*p, Color::YELLOW);
                }
            }
        }
    }

    fn split(&mut self) {
        let mid_x = (self.from.x + self.to.x) / 2.0;
        let mid_y = (self.from.y + self.to.y) / 2.0;

        let mut inner = TreeState::Inner(Box::new([
            Self::new(self.from, Point::new(mid_x, mid_y)), // top left
            Self::new(Point::new(mid_x, self.from.y), Point::new(self.to.x, mid_y)), // top right
            Self::new(Point::new(self.from.x, mid_y), Point::new(mid_x, self.to.y)), // bottom left
            Self::new(Point::new(mid_x, mid_y), self.to),   // bottom right
        ]));
        std::mem::swap(&mut inner, &mut self.tree);
    }

    fn fmt_impl(&self, f: &mut std::fmt::Formatter, depth: i32) -> std::fmt::Result {
        let indent = (0..depth * 2).map(|_| " ").collect::<String>();
        match &self.tree {
            TreeState::Leaf(None) => write!(f, "{}Empty {} - {}", indent, self.from, self.to),
            TreeState::Leaf(Some(p)) => {
                write!(f, "{}Leaf {} - {} = {}", indent, self.from, self.to, p)
            }
            TreeState::Inner(children) => {
                write!(f, "{}QuadTree {} - {}", indent, self.from, self.to)?;
                for child in children.iter() {
                    writeln!(f)?;
                    child.fmt_impl(f, depth + 1)?;
                }
                Ok(())
            }
        }
    }
}

impl std::fmt::Display for QuadTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.fmt_impl(f, 0)
    }
}

struct ElectionMap {
    result: Vec<u8>,
    width: usize,
    height: usize,
}

impl ElectionMap {
    fn new(width: usize, height: usize) -> ElectionMap {
        ElectionMap {
            result: vec![0; width * height],
            width,
            height,
        }
    }

    fn set_result(&mut self, from: Point, to: Point, winner: usize) {
        self.set_rect(from, to, winner as u8 + 1);
    }

    fn has_result(&self, _from: Point, to: Point) -> bool {
        let from_x = (to.x * (self.width - 1) as f32).round() as usize;
        let from_y = (to.y * (self.height - 1) as f32).round() as usize;
        self.result[self.as_index(from_x, from_y)] > 0
    }

    fn clear(&mut self, from: Point, to: Point) {
        self.set_rect(from, to, 0);
    }

    fn set_rect(&mut self, from: Point, to: Point, value: u8) {
        let from_x = (from.x * (self.width - 1) as f32).round() as usize;
        let from_y = (from.y * (self.height - 1) as f32).round() as usize;
        let to_x = (to.x * (self.width - 1) as f32).round() as usize;
        let to_y = (to.y * (self.height - 1) as f32).round() as usize;
        for x in from_x..=to_x {
            for y in from_y..=to_y {
                let i = self.as_index(x, y);
                self.result[i] = value;
            }
        }
    }

    fn same_result_as_neighbours(&self, from: Point, to: Point) -> bool {
        // We only need to check the immediate border of the rectangle.
        let from_x = (from.x * (self.width - 1) as f32).round() as usize;
        let from_y = (from.y * (self.height - 1) as f32).round() as usize;
        let to_x = (to.x * (self.width - 1) as f32).round() as usize;
        let to_y = (to.y * (self.height - 1) as f32).round() as usize;

        let result = self.result[self.as_index(from_x, from_y)];

        for x in from_x..to_x {
            if from_y > 0 && self.result[self.as_index(x, from_y - 1)] != result {
                return false;
            }
            if to_y + 1 < self.width && self.result[self.as_index(x, to_y + 1)] != result {
                return false;
            }
        }
        for y in from_y..to_y {
            if from_x > 0 && self.result[self.as_index(from_x - 1, y)] != result {
                return false;
            }
            if to_x + 1 < self.height && self.result[self.as_index(to_x + 1, y)] != result {
                return false;
            }
        }
        true
    }

    fn draw(&self, image: &mut Image, colors: &[Color]) {
        for x in 0..self.width {
            for y in 0..self.height {
                let result = (self.result[self.as_index(x, y)] - 1) as usize;
                let color = if result < colors.len() {
                    colors[result]
                } else {
                    Color::BLACK
                };
                image.set_coords(x, y, color);
            }
        }
    }

    fn as_index(&self, x: usize, y: usize) -> usize {
        x * self.height + y
    }
}

fn normal_pdf(mean: f32, sigma: f32, x: f32) -> f32 {
    1f32 / (sigma * (2f32 * std::f32::consts::PI).sqrt())
        * (-1f32 / 2f32 * ((x - mean) / sigma).powi(2)).exp()
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
            let at = Point::new(p.x + dx, p.y + dy);
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
        Point::new(0.5, 0.99),
        Point::new(0.07, 0.25),
        Point::new(0.93, 0.25),
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

    let colors = vec![Color::RED, Color::GREEN, Color::BLUE];

    log!("sample points: {:?}", sample_points);

    let mut tree = QuadTree::default();
    for p in candidates.iter() {
        let winner = run_election(*p, &candidates, &sample_points);
        log!("at {:?} elected: {:}", p, winner);
        tree.insert(*p);
    }

    log!("built tree: {}", tree);
    let mut map = ElectionMap::new(width, height);

    let num_steps = 6;
    for i in 0..num_steps {
        let mut num_elections = 0;
        // Run an election for each leaf of the tree.
        tree.visit_leaves(&mut |from: Point, to: Point| {
            if !map.has_result(from, to) {
                let mid = from + (to - from) * 0.5;
                let winner = run_election(mid, &candidates, &sample_points);
                // log!("at {:?} elected: {:}", mid, winner);
                map.set_result(from, to, winner);
                num_elections += 1;
            }
        });

        // Split all leaves whose neighbours don't all have the same election result.
        let mut to_split = vec![];
        tree.visit_leaves(&mut |from: Point, to: Point| {
            if !map.same_result_as_neighbours(from, to) {
                // log!("need to split {}-{}", from, to);
                to_split.push((from, to));
            }
        });

        let last_step = i + 1 == num_steps;

        if !last_step {
            for (from, to) in to_split {
                map.clear(from, to);
                let dx = Vec2::new(to.x - from.x, 0.0);
                let dy = Vec2::new(0.0, to.y - from.y);
                tree.insert(from + dx * 0.25 + dy * 0.25);
                tree.insert(from + dx * 0.25 + dy * 0.75);
                tree.insert(from + dx * 0.75 + dy * 0.25);
                tree.insert(from + dx * 0.75 + dy * 0.75);
            }
        }
        // Repeat until minimum leave size reached or no more splits are necessary.

        log!("step {}: {} elections", i, num_elections);
    }

    let mut image = Image::new(width, height);
    map.draw(&mut image, &colors);
    // tree.draw_outlines(&mut image);

    Ok(image.data)
}
