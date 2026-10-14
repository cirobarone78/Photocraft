//! Inspect the real global solver on a local guide fixture, without modifying its image.
//! Usage: cargo run -p photocraft-algo --example global_guides -- fixture.json output.json
use photocraft_algo::global_perspective::{Guide, Options, solve};
use photocraft_geom::Rect;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    image_size: [i32; 2],
    guides: Vec<Guide>,
    #[serde(default)]
    options: Options,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let input = args.next().ok_or("pass a fixture JSON path")?;
    let output = args.next().ok_or("pass a new output JSON path")?;
    if std::fs::metadata(&input)?.len() > 262_144 {
        return Err("guide fixture exceeds 256 KiB".into());
    }
    let fixture: Fixture = serde_json::from_str(&std::fs::read_to_string(&input)?)?;
    let [w, h] = fixture.image_size;
    if w <= 0 || h <= 0 {
        return Err("invalid image size".into());
    }
    let start = std::time::Instant::now();
    let solution = solve(&fixture.guides, Rect::new(0, 0, w, h), &fixture.options)?;
    let result = serde_json::json!({"solution":solution,"solveMs":start.elapsed().as_secs_f64()*1000.0});
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(output)?;
    serde_json::to_writer_pretty(&mut file, &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
