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
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const PINK: Color = Color {
        r: 255,
        g: 20,
        b: 147,
    };
}
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    fn l2_square(&self, other: &Point) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
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

fn vote_plurality(p: Point, candidates: &[Point], votes: &mut [u8]) {
    let mut closest_i = 100000000;
    let mut closest_dist = std::f32::MAX;
    for (i, c) in candidates.iter().enumerate() {
        let dist = p.l2_square(c);
        if dist < closest_dist {
            closest_dist = dist;
            closest_i = i;
        }
    }
    votes[closest_i] = 1;
}

fn vote_close(p: Point, candidates: &[Point], votes: &mut [u8]) {
    for (i, c) in candidates.iter().enumerate() {
        let dist = p.l2_square(c);
        if dist < 1.0 {
            votes[i] = 1;
        }
    }
}

fn vote_rank(p: Point, candidates: &[Point], votes: &mut [u8]) {
    let mut min_dist = std::f32::MIN;
    let mut prev_i = 1000000;
    for rank in 0..candidates.len() {
        let mut closest_i = 100000000;
        let mut closest_dist = std::f32::MAX;
        for (i, c) in candidates.iter().enumerate() {
            let dist = p.l2_square(c);
            if dist < closest_dist && dist >= min_dist && i != prev_i {
                closest_dist = dist;
                closest_i = i;
            }
        }
        min_dist = closest_dist;
        prev_i = closest_i;
        votes[closest_i] = (1 + rank) as u8;
    }
}

fn max_vote_candidate(votes: &[f32]) -> usize {
    votes
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap()
}

fn min_vote_candidate(votes: &[f32]) -> usize {
    votes
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap()
}

pub const CANDIDATE_COLORS: [Color; 5] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::YELLOW,
    Color::PINK,
];

#[wasm_bindgen]
pub fn render(
    size: usize,
    candidate_coords: Vec<f32>,
    election_method: &str,
) -> Result<Vec<u8>, JsValue> {
    utils::set_panic_hook();

    let mut candidates = vec![];
    for i in (0..candidate_coords.len()).step_by(2) {
        candidates.push(Point::new(candidate_coords[i], candidate_coords[i + 1]));
    }
    let winners = election(size as i32, &candidates, election_method);
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

#[wasm_bindgen]
pub fn candidate_color(i: usize) -> String {
    let color = CANDIDATE_COLORS[i];
    format!("rgb({}, {}, {})", color.r, color.g, color.b)
}

fn compute_votes<F>(
    size: i32,
    start: i32,
    end: i32,
    candidates: &[Point],
    voting_method: F,
) -> Vec<u8>
where
    F: Fn(Point, &[Point], &mut [u8]),
{
    let padded_size = (end - start) as i32;
    let mut results = vec![0u8; padded_size.pow(2) as usize * candidates.len()];

    // Compute voting results at each individual point.
    for x in start..end {
        for y in start..end {
            let at = Point::new(x as f32 / size as f32, y as f32 / size as f32);

            let i = x - start;
            let j = y - start;
            let offset = ((i * padded_size + j) as usize) * candidates.len();

            voting_method(
                at,
                &candidates,
                &mut results[offset..offset + candidates.len()],
            );
        }
    }

    results
}

fn declare_winner<F>(
    size: i32,
    num_votes: &[f32],
    candidates: &[Point],
    select_winner: F,
) -> Vec<u8>
where
    F: Fn(&[f32]) -> usize,
{
    let mut winners = vec![0u8; size.pow(2) as usize];
    for x in 0..size {
        for y in 0..size {
            let i = ((x * size) + y) as usize * candidates.len();
            let winner = select_winner(&num_votes[i..i + candidates.len()]);
            winners[(x * size + y) as usize] = winner as u8;
        }
    }
    winners
}

fn sum_votes<F>(
    size: i32,
    candidates: &[Point],
    start: i32,
    end: i32,
    results: &[u8],
    sample_locations: &[(i32, f32)],
    count_votes: F,
) -> Vec<f32>
where
    F: Fn(&mut [f32], &[u8], f32),
{
    let padded_size = (end - start) as i32;

    // Sum up all the votes for the neighbour of each point.
    let mut num_votes = vec![0f32; size.pow(2) as usize * candidates.len()];
    for x in 0..size {
        for y in start..end {
            // Sum up all the votes along the x-neighbourhood.
            let mut line_votes = vec![0f32; candidates.len()];
            for (dx, p) in sample_locations.iter() {
                let i = x + dx - start;
                let j = y - start;
                let offset = ((i * padded_size + j) as usize) * candidates.len();

                count_votes(
                    &mut line_votes,
                    &results[offset..offset + candidates.len()],
                    *p,
                );
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
    num_votes
}

pub fn election(size: i32, candidates: &[Point], election_method: &str) -> Vec<u8> {
    let sigma = 0.5f32 / 1.5;
    let num_sigma = 3.0;

    // The vote map is [0; 1], so we need to compute votes in [-num_sigma * sigma; 1 + num_sigma * sigma].
    let range = (size as f32 * sigma * num_sigma) as i32;
    let start = -range;
    let end = size + range;

    // Compute voting results at each individual point.
    let results = match election_method {
        "plurality" => compute_votes(size, start, end, &candidates, vote_plurality),
        "approval" => compute_votes(size, start, end, &candidates, vote_close),
        "borda" | "condorcet" | "hare" => compute_votes(size, start, end, &candidates, vote_rank),
        _ => unreachable!("unsupported election method {}", election_method),
    };

    // Neighbourhood weighting.
    let mut sample_locations = vec![];
    for x in -range..range {
        let p = normal_pdf(0f32, sigma, x as f32 / size as f32);
        sample_locations.push((x, p));
    }

    // Sum up all votes weighted by their neighborhouds.
    let num_votes = match election_method {
        "hare" => sum_votes(
            size,
            candidates,
            start,
            end,
            &results,
            &sample_locations,
            |line_votes, results, p| {
                let mut winner = 0;
                let mut best_rank = 255;
                for c in 0..candidates.len() {
                    let rank = results[c];
                    if rank < best_rank {
                        best_rank = rank;
                        winner = c;
                    }
                }
                line_votes[winner] += p;
            },
        ),
        _ => sum_votes(
            size,
            candidates,
            start,
            end,
            &results,
            &sample_locations,
            |line_votes, results, p| {
                for c in 0..candidates.len() {
                    line_votes[c] += results[c] as f32 * p;
                }
            },
        ),
    };

    if election_method == "hare" {
        let undecided = 255;
        let mut winners = vec![undecided; size.pow(2) as usize];

        let mut votes_with_eliminated_candidates = vec![None; 2usize.pow(candidates.len() as u32)];
        for x in 0..size {
            for y in 0..size {
                // First, check if we already have a majority winner.
                let vote_i = ((x * size) + y) as usize * candidates.len();
                let votes = &num_votes[vote_i..vote_i + candidates.len()];

                let maybe_winner = max_vote_candidate(&votes);
                let vote_sum: f32 = votes.iter().sum();
                if votes[maybe_winner] >= 0.5 * vote_sum {
                    // If one candidate has more than half the ballots, that candidate wins.
                    winners[(x * size + y) as usize] = maybe_winner as u8;
                    continue;
                }

                // Otherwise, the candidate with the fewest ballots is eliminated and we vote again.
                let mut eliminated = 1 << min_vote_candidate(&votes);

                for _ in 0..candidates.len() {
                    let num_votes = votes_with_eliminated_candidates[eliminated]
                        .get_or_insert_with(|| {
                            sum_votes(
                                size,
                                candidates,
                                start,
                                end,
                                &results,
                                &sample_locations,
                                |line_votes, results, p| {
                                    let mut winner = 0;
                                    let mut best_rank = 255;
                                    for c in 0..candidates.len() {
                                        let rank = results[c];
                                        if rank < best_rank && (1 << c) & eliminated == 0 {
                                            best_rank = rank;
                                            winner = c;
                                        }
                                    }
                                    line_votes[winner] += p;
                                },
                            )
                        });
                    let votes = &num_votes[vote_i..vote_i + candidates.len()];

                    // Check if we have a winner.
                    let maybe_winner = max_vote_candidate(&votes);
                    let vote_sum: f32 = votes.iter().sum();
                    if votes[maybe_winner] >= 0.5 * vote_sum {
                        // If one candidate has more than half the ballots, that candidate wins.
                        winners[(x * size + y) as usize] = maybe_winner as u8;
                        break;
                    } else {
                        // Otherwise, the candidate with the fewest ballots is eliminated.
                        let mut worst_candidate = 255;
                        let mut min_votes = 1e9;
                        for c in 0..candidates.len() {
                            let v = votes[c];
                            if v < min_votes && (1 << c) & eliminated == 0 {
                                min_votes = v;
                                worst_candidate = c;
                            }
                        }
                        eliminated |= 1 << worst_candidate;
                    }
                }
            }
        }

        winners
    } else {
        // Select the winner of the election for each point.
        match election_method {
            "plurality" | "approval" => {
                declare_winner(size, &num_votes, candidates, max_vote_candidate)
            }
            "borda" => declare_winner(size, &num_votes, candidates, min_vote_candidate),
            "condorcet" => unimplemented!(
                "condorcet; must select candidate that would defeat all others one-to-one"
            ),
            _ => unreachable!("unsupported election method {}", election_method),
        }
    }
}
