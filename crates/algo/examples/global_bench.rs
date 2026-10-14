//! Bounded 24 MP release benchmark of legacy mesh vs exact global inverse rendering.
//! Both paths receive equivalent global geometry; this is not a quality claim for V2.
//! RAYON_NUM_THREADS=2 cargo run --release -p photocraft-algo --example global_bench -- new.json
use photocraft_algo::global_perspective::{Guide, Options, solve};
use photocraft_algo::perspective::{PerspectiveMap, Plane};
use photocraft_algo::transform::{Homography, Interp, warp_surface};
use photocraft_algo::warp::warp_mesh_surface;
use photocraft_color::PixelFormat;
use photocraft_geom::Rect;
use photocraft_raster::Surface;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1).ok_or("pass a new result JSON path")?;
    if std::path::Path::new(&output).exists() {
        return Err("benchmark output already exists".into());
    }
    let bounds = Rect::new(0, 0, 6000, 4000);
    let projection = Homography([0.92, 0.025, 80.0, -0.035, 0.90, 110.0, 0.000015, -0.000025, 1.0]);
    let mut guides = Vec::new();
    for i in 0..6 {
        let x = 550.0 + (i % 2) as f64 * 3000.0;
        let y = 450.0 + (i / 2) as f64 * 1000.0;
        let q = [[x, y], [x + 650.0, y], [x + 650.0, y + 700.0], [x, y + 700.0]].map(|p| {
            let (x, y) = projection.apply(p[0], p[1]);
            [x, y]
        });
        guides.push(Guide { id: i, corners: q, enabled: true, weight: 1.0 });
    }
    let start = Instant::now();
    let solution = solve(&guides, bounds, &Options::default())?;
    let solve_ms = start.elapsed().as_secs_f64() * 1000.0;
    let h = solution.homography();
    let planes: Vec<Plane> = guides
        .iter()
        .map(|g| Plane {
            src: g.corners,
            dst: g.corners.map(|p| {
                let (x, y) = h.apply(p[0], p[1]);
                [x, y]
            }),
        })
        .collect();
    let legacy = PerspectiveMap::new(&planes).ok_or("legacy comparison map failed")?;
    let mut max_geometry_error = 0.0_f64;
    for y in (0..=4000).step_by(100) {
        for x in (0..=6000).step_by(100) {
            let a = h.apply(x as f64, y as f64);
            let b = legacy.map(x as f64, y as f64);
            max_geometry_error = max_geometry_error.max((a.0 - b.0).hypot(a.1 - b.1));
        }
    }
    if max_geometry_error > 1e-6 {
        return Err("benchmark paths do not have equivalent geometry".into());
    }
    let mut src = Surface::new(PixelFormat::RGBA8);
    src.fill_rect(bounds, &[0.6, 0.5, 0.4, 1.0]);
    for y in (0..4000).step_by(120) {
        src.fill_rect(Rect::new(0, y, 6000, y + 3), &[0.1, 0.2, 0.3, 1.0]);
    }
    for x in (0..6000).step_by(200) {
        src.fill_rect(Rect::new(x, 0, x + 3, 4000), &[0.8, 0.7, 0.5, 1.0]);
    }
    let start = Instant::now();
    let old = warp_mesh_surface(&src, bounds, &|x, y| legacy.map(x, y), Interp::Bicubic);
    let legacy_ms = start.elapsed().as_secs_f64() * 1000.0;
    let old_sample = old.pixel(3000, 2000);
    drop(old);
    let start = Instant::now();
    let new = warp_surface(&src, bounds, &h, Interp::Bicubic)?;
    let global_ms = start.elapsed().as_secs_f64() * 1000.0;
    let new_sample = new.pixel(3000, 2000);
    let result = serde_json::json!({"fixture":"originalSynthetic24MP","width":6000,"height":4000,"guides":6,
        "releaseBuild":!cfg!(debug_assertions),"rayonThreadsEnvironment":std::env::var("RAYON_NUM_THREADS").ok(),
        "solveMs":solve_ms,"legacyEquivalentGeometryMeshMs":legacy_ms,"globalInverseRenderMs":global_ms,
        "legacyOverGlobalRatio":legacy_ms/global_ms,"maxGeometryDifferencePx":max_geometry_error,
        "oldCenterSample":old_sample,"newCenterSample":new_sample,"outputBounds":solution.output_bounds,
        "note":"Single measurement on identical source/geometry, different render algorithms. Not a general performance guarantee or a V2 quality approval."});
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(output)?;
    serde_json::to_writer_pretty(&mut file, &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
