use image::GenericImageView;
use voting_map::{election, Point, CANDIDATE_COLORS};

fn assert_image(name: &str, candidate: &image::DynamicImage) {
    std::fs::create_dir_all("test_output").expect("failed to create test_output");
    let candidate_path = format!("test_output/{}.png", name);
    std::fs::remove_file(&candidate_path);
    candidate
        .save(&candidate_path)
        .expect("failed to write candidate image");

    let golden_path = format!("testdata/{}.png", name);

    // candidate.save(&golden_path);

    let golden = image::open(&golden_path);
    assert!(
        golden.is_ok(),
        "Failed to find reference image: {:}. If this is the first time you run this test, please provide a reference image or copy the candidate from test_output/ after inspecting it to ensure that it is correct.",
        golden_path
    );
    let golden = golden.unwrap();

    assert_eq!(candidate.height(), golden.height());
    assert_eq!(candidate.width(), golden.width());
    if candidate.to_bytes() == golden.to_bytes() {
        // Images match, nothing to do.
        return;
    }

    // Images don't match, create a diff to make comparisons easier.
    let mut num_diffs = 0;
    let diff = image::ImageBuffer::from_fn(golden.width(), golden.height(), |x, y| {
        let got = candidate.get_pixel(x, y);
        let want = golden.get_pixel(x, y);
        if got == want {
            image::Rgba([0, 0, 0, 0])
        } else {
            num_diffs += 1;
            got
        }
    });
    image::DynamicImage::ImageRgba8(diff)
        .save(format!("test_output/{}_diff.png", name))
        .expect("failed to write diff");
    panic!("Image differs in {} pixels", num_diffs);
}

fn get_candidates(name: &str) -> Vec<Point> {
    if name == "equilateral" {
        vec![
            Point::new(0.5, 0.99),
            Point::new(0.07, 0.25),
            Point::new(0.93, 0.25),
        ]
    } else if name == "squeezed" {
        vec![
            Point::new(0.07, 0.17),
            Point::new(0.49, 0.01),
            Point::new(0.41, 0.02),
        ]
    } else if name == "split" {
        vec![
            Point::new(0.93, 0.49),
            Point::new(0.79, 0.42),
            Point::new(0.27, 0.45),
        ]
    } else if name == "nonmonotonic" {
        vec![
            Point::new(0.54, 0.47),
            Point::new(0.77, 0.64),
            Point::new(0.13, 0.10),
        ]
    } else if name == "square" {
        vec![
            Point::new(0.3, 0.3),
            Point::new(0.3, 0.7),
            Point::new(0.7, 0.7),
            Point::new(0.7, 0.3),
        ]
    } else if name == "shattered" {
        vec![
            Point::new(0.12, 0.28),
            Point::new(0.39, 0.28),
            Point::new(0.97, 0.14),
            Point::new(0.85, 0.70),
        ]
    } else if name == "disjoint" {
        vec![
            Point::new(0.24, 0.25),
            Point::new(0.04, 0.64),
            Point::new(0.85, 0.55),
            Point::new(0.19, 0.62),
        ]
    } else if name == "nonmonotonicity" {
        vec![
            Point::new(0.4, 0.57),
            Point::new(0.05, 0.62),
            Point::new(0.91, 0.7),
            Point::new(0.16, 0.54),
        ]
    } else {
        panic!("unknown candidate set: {}", name);
    }
}

fn assert_election(method: &str, candidate_name: &str, size: u32) {
    let candidates = get_candidates(candidate_name);
    let mut tranformed = vec![];
    for c in candidates {
        // Scale to [-0.25, 1.25] coordinates and flip y-axis to match http://zesty.ca/voting/sim/
        tranformed.push(Point::new((c.x + 0.25) / 1.5, (1.0 - c.y + 0.25) / 1.5));
    }
    let winners = election(size as i32, &tranformed, method);

    let got = image::ImageBuffer::from_fn(size, size, |x, y| {
        let c = CANDIDATE_COLORS[winners[(x * size + y) as usize] as usize];
        image::Rgb([c.r, c.g, c.b])
    });

    assert_image(
        &format!("{}_{}", method, candidate_name),
        &image::DynamicImage::ImageRgb8(got),
    );
}

#[test]
fn plurality_equilateral() {
    assert_election("plurality", "equilateral", 256);
}

#[test]
fn plurality_squeezed() {
    assert_election("plurality", "squeezed", 256);
}

#[test]
fn plurality_split() {
    assert_election("plurality", "split", 256);
}

#[test]
fn plurality_nonmonotonic() {
    assert_election("plurality", "nonmonotonic", 256);
}

#[test]
fn borda_equilateral() {
    assert_election("borda", "equilateral", 256);
}

#[test]
fn borda_squeezed() {
    assert_election("borda", "squeezed", 256);
}

#[test]
fn borda_split() {
    assert_election("borda", "split", 256);
}

#[test]
fn borda_nonmonotonic() {
    assert_election("borda", "nonmonotonic", 256);
}

#[test]
fn approval_equilateral() {
    assert_election("approval", "equilateral", 256);
}

#[test]
fn approval_squeezed() {
    assert_election("approval", "squeezed", 256);
}

#[test]
fn approval_split() {
    assert_election("approval", "split", 256);
}

#[test]
fn approval_nonmonotonic() {
    assert_election("approval", "nonmonotonic", 256);
}

#[test]
fn hare_equilateral() {
    assert_election("hare", "equilateral", 64);
}

#[test]
fn hare_squeezed() {
    assert_election("hare", "squeezed", 64);
}

#[test]
fn hare_split() {
    assert_election("hare", "split", 64);
}

#[test]
fn hare_nonmonotonic() {
    assert_election("hare", "nonmonotonic", 64);
}

#[test]
fn hare_square() {
    assert_election("hare", "square", 96);
}

#[test]
fn hare_shattered() {
    assert_election("hare", "shattered", 96);
}

#[test]
fn hare_disjoint() {
    assert_election("hare", "disjoint", 96);
}

#[test]
fn hare_nonmonotonicity() {
    assert_election("hare", "nonmonotonicity", 96);
}
