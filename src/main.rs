use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

use crate::{
    calculator::{calc_beam_points, iter_elevations, ElevationRange},
    plotter::{create_grid_lines, PlotAxisConfig, PlotAxisName},
};

#[derive(Properties, PartialEq)]
pub struct BeamViewerProps {
    pub lat_deg: f64,
    pub alt_meter: f64,
}

#[function_component(BeamViewer)]
fn beam_viewer(BeamViewerProps { lat_deg, alt_meter }: &BeamViewerProps) -> Html {
    let el_ranges = vec![
        ElevationRange::new(0, 2),
        ElevationRange::new(50, 5),
        ElevationRange::new(100, 10),
        ElevationRange::new(150, 50),
        ElevationRange::new(450, 0),
    ];
    let el_highlights = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20, 30]
        .into_iter()
        .map(|i| i as f64)
        .collect::<Vec<_>>();

    let max_range_meter = 300_000_f64;
    let max_alt_meter = 15_000_f64;
    let n_range_section = 100;

    let polylines = iter_elevations(&el_ranges)
        .map(|el| {
            let beam_points =
                calc_beam_points(&max_range_meter, &n_range_section, &el, lat_deg, alt_meter);
            let highlighted = el_highlights.contains(&el);
            create_polyline_for_beam(beam_points, highlighted)
        })
        .collect::<Html>();

    let plot_size = 1000_f64;
    let margin_size = 150_f64;
    let aspect_ratio = max_range_meter / max_alt_meter;
    let inner_height = plot_size / aspect_ratio;

    let axis_label_distance = 75_f64;
    let x_label_loc = (
        margin_size + plot_size / 2_f64,
        margin_size + plot_size + axis_label_distance,
    );
    let y_label_loc = (
        margin_size - axis_label_distance,
        margin_size + plot_size / 2_f64,
    );

    let tick_label_distance = 20_f64;
    let inner_height = format!("{:.0}", inner_height);
    let inner_width = format!("{:.0}", plot_size);
    let inner_view_box = format!("0 0 {} {}", max_range_meter, max_alt_meter);
    let transform = format!(
        "translate({} {}) scale(1 -{:.0}) translate(0 -{})",
        margin_size, margin_size, aspect_ratio, inner_height
    );
    let outer_size = plot_size + margin_size * 2_f64;
    let outer_view_box = format!("0 0 {:.0} {:.0}", outer_size, outer_size);
    let axis1 = PlotAxisConfig::new(PlotAxisName::X, 0, max_range_meter as u32, 50_000, 10_000);
    let axis2 = PlotAxisConfig::new(PlotAxisName::Y, 0, max_alt_meter as u32, 5_000, 1_000);
    let grid_lines = create_grid_lines(&axis1, &axis2);
    let x_label_loc_x = format!("{:.0}", x_label_loc.0);
    let x_label_loc_y = format!("{:.0}", x_label_loc.1);
    let y_label_loc_x = format!("{:.0}", y_label_loc.0);
    let y_label_loc_y = format!("{:.0}", y_label_loc.1);
    let tick_labels_axis1 =
        create_tick_labels(&axis1, &margin_size, &plot_size, &tick_label_distance);
    let tick_labels_axis2 =
        create_tick_labels(&axis2, &margin_size, &plot_size, &tick_label_distance);

    html! {
        <svg id="viewer" viewBox={ outer_view_box }>
            <g transform={ transform }>
                <svg viewBox={ inner_view_box } width={ inner_width } height={ inner_height }>
                    { grid_lines }
                    { polylines }
                    <rect width="100%" height="100%" class="frame" />
                </svg>
            </g>
            <text
                class="axis-label"
                x={ x_label_loc_x }
                y={ x_label_loc_y }
                text-anchor="middle"
                dominant-baseline="hanging"
            >
                { "Distance (km)" }
            </text>
            <text
                class="axis-label y-axis"
                x={ y_label_loc_x }
                y={ y_label_loc_y }
                text-anchor="middle"
                dominant-baseline="auto"
            >
                { "Altitude (km)" }
            </text>
            { tick_labels_axis1 }
            { tick_labels_axis2 }
        </svg>
    }
}

fn create_polyline_for_beam(
    points: impl Iterator<Item = calculator::AtmosphericPoint>,
    highlighted: bool,
) -> Html {
    let polygon_points = points
        .map(|point| format!("{:.0}, {:.0}", point.dist_meter, point.alt_meter))
        .collect::<Vec<_>>()
        .join(" ");
    let additional_class = if highlighted { " highlighted" } else { "" };
    let class_names = format!("beam-curve{}", additional_class);
    html! {
        <polyline points={ polygon_points } class={ class_names }/>
    }
}

fn create_tick_labels(
    axis: &PlotAxisConfig,
    margin_size: &f64,
    plot_size: &f64,
    tick_label_distance: &f64,
) -> Html {
    let factor = plot_size / (axis.end - axis.start) as f64;
    let origin = match axis.name {
        PlotAxisName::X => (*margin_size, margin_size + plot_size + tick_label_distance),
        PlotAxisName::Y => (margin_size - tick_label_distance, margin_size + plot_size),
    };

    (axis.start..=axis.end)
        .step_by(axis.major_step as usize)
        .map(|val| {
            let inc = val as f64 * factor;
            let coord = match axis.name {
                PlotAxisName::X => (origin.0 + inc, origin.1),
                PlotAxisName::Y => (origin.0, origin.1 - inc),
            };
            let km = val / 1000;
            create_tick_label(km, coord, &axis.name)
        })
        .collect::<Html>()
}

fn create_tick_label(val: u32, (x, y): (f64, f64), axis: &PlotAxisName) -> Html {
    let x = format!("{:.0}", x);
    let y = format!("{:.0}", y);
    match axis {
        PlotAxisName::X => html! {
            <text
                class="tick-label x-axis"
                x={ x }
                y={ y }
                text-anchor="middle"
                dominant-baseline="hanging"
            >
                { val }
            </text>
        },
        PlotAxisName::Y => html! {
            <text
                class="tick-label y-axis"
                x={ x }
                y={ y }
                text-anchor="end"
                dominant-baseline="middle"
            >
                { val }
            </text>
        },
    }
}

#[function_component(App)]
fn app() -> Html {
    let alt_meter_handle = use_state(|| 0.0_f64);
    let alt_meter = *alt_meter_handle;
    let lat_deg_handle = use_state(|| 0.0_f64);
    let lat_deg = *lat_deg_handle;

    let on_alt_change = Callback::from(move |e: Event| {
        let target: EventTarget = e.target().expect("unknown event target");
        let value = target.unchecked_into::<HtmlInputElement>().value();
        if let Ok(value) = value.parse() {
            alt_meter_handle.set(value);
        }
        // if value is not numeric, just ignore.
    });

    let on_lat_change = Callback::from(move |e: Event| {
        let target: EventTarget = e.target().expect("unknown event target");
        let value = target.unchecked_into::<HtmlInputElement>().value();
        if let Ok(value) = value.parse() {
            lat_deg_handle.set(value);
        }
        // if value is not numeric, just ignore.
    });

    let alt_meter_value = format!("{}", alt_meter);
    let lat_deg_value = format!("{}", lat_deg);

    html! {
        <>
            <h1>{ "Radar beam path calculator" }</h1>
            <div>
                <label for="radar-altitude">{"Altitude (m)"}</label>
                <input onchange={ on_alt_change }
                    id="radar-altitude"
                    type="number"
                    value={ alt_meter_value }
                />
                <br/>
                <label for="radar-latitude">{"Latitude (deg)"}</label>
                <input onchange={ on_lat_change }
                    id="radar-latitude"
                    type="number"
                    value={ lat_deg_value }
                />
            </div>
            <BeamViewer lat_deg={ lat_deg } alt_meter={ alt_meter } />
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

mod calculator;
mod plotter;
