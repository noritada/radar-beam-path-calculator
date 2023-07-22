use crate::calculator::{calc_beam_points, iter_elevations, ElevationRange};

fn main() {
    let el_ranges = vec![
        ElevationRange::new(0, 2),
        ElevationRange::new(50, 5),
        ElevationRange::new(100, 10),
        ElevationRange::new(150, 50),
        ElevationRange::new(450, 0),
    ];

    let max_range_meter = 60_000_f64;
    let lat_deg = 36.0;
    let alt_meter = 0.0;
    let n_range_section = 100;

    let beams = iter_elevations(&el_ranges)
        .map(|el| {
            calc_beam_points(
                &max_range_meter,
                &n_range_section,
                &el,
                &lat_deg,
                &alt_meter,
            )
            .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    dbg!(beams);
}

mod calculator;
