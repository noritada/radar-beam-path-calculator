use yew::prelude::*;

use crate::{
    calculator::{calc_beam_points, iter_elevations, ElevationRange},
    plotter::{create_grid_lines, PlotAxisConfig, PlotAxisName},
};

#[function_component(BeamViewer)]
fn beam_viewer() -> Html {
    let el_ranges = vec![
        ElevationRange::new(0, 2),
        ElevationRange::new(50, 5),
        ElevationRange::new(100, 10),
        ElevationRange::new(150, 50),
        ElevationRange::new(450, 0),
    ];

    let max_range_meter = 300_000_f64;
    let lat_deg = 36.0;
    let alt_meter = 0.0;
    let n_range_section = 100;

    let polylines = iter_elevations(&el_ranges)
        .map(|el| {
            let beam_points = calc_beam_points(
                &max_range_meter,
                &n_range_section,
                &el,
                &lat_deg,
                &alt_meter,
            );
            create_polyline_for_beam(beam_points)
        })
        .collect::<Html>();

    let width = format!("{:.0}", max_range_meter);
    let view_box = format!("0 0 {} 15000", width);
    let axis1 = PlotAxisConfig::new(PlotAxisName::X, 0, max_range_meter as u32, 50_000, 10_000);
    let axis2 = PlotAxisConfig::new(PlotAxisName::Y, 0, 15_000, 5_000, 1_000);
    let grid_lines = create_grid_lines(&axis1, &axis2);

    html! {
        <svg id="viewer">
            <g transform="scale(0.8, -6)" transform-origin="center">
                <svg viewBox={ view_box }>
                    { grid_lines }
                    { polylines }
                    <rect width="100%" height="100%" class="frame" />
                </svg>
            </g>
        </svg>
    }
}

fn create_polyline_for_beam(points: impl Iterator<Item = calculator::AtmosphericPoint>) -> Html {
    let polygon_points = points
        .map(|point| format!("{:.0}, {:.0}", point.dist_meter, point.alt_meter))
        .collect::<Vec<_>>()
        .join(" ");
    html! {
        <polyline points={ polygon_points } class="beam-curve"/>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <>
            <h1>{ "Radar beam calculator" }</h1>
            <BeamViewer/>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

mod calculator;
mod plotter;
