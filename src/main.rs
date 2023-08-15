use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

use crate::{
    calculator::{calc_beam_points, iter_elevations, ElevationRange},
    plotter::{create_grid_lines, LabelLoc, LabelPlotter, PlotAxisConfig, PlotAxisName},
};

#[derive(Properties, PartialEq)]
pub struct BeamViewerProps {
    pub lat_deg: f64,
    pub alt_meter: f64,
    pub distance_axis: DistanceAxis,
}

#[function_component(BeamViewer)]
fn beam_viewer(
    BeamViewerProps {
        lat_deg,
        alt_meter,
        distance_axis,
    }: &BeamViewerProps,
) -> Html {
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

    let max_range_meter = distance_axis.max as f64 * 1_000_f64;
    let max_alt_meter = 15_000_f64;
    let n_range_section = 100;

    let el_beam_points = iter_elevations(&el_ranges)
        .map(|el| {
            let beam_points = calc_beam_points(
                &(max_range_meter + max_alt_meter),
                &n_range_section,
                &el,
                lat_deg,
                alt_meter,
            )
            .collect::<Vec<_>>();
            let highlighted = el_highlights.contains(&el);
            (el, highlighted, beam_points)
        })
        .collect::<Vec<_>>();
    let polylines = el_beam_points
        .iter()
        .map(|(_el, highlighted, points)| create_polyline_for_beam(points.iter(), *highlighted))
        .collect::<Html>();
    let el_labels = el_beam_points
        .iter()
        .filter_map(|(el, highlighted, points)| {
            if *highlighted {
                let loc = locate_beam_labels(points.iter(), &max_range_meter, &max_alt_meter);
                Some((el, loc))
            } else {
                None
            }
        });

    let plot_size = 1000_f64;
    let margin_size = 150_f64;
    let aspect_ratio = max_range_meter / max_alt_meter;
    let inner_height = plot_size / aspect_ratio;

    let axis_label_distance = 75_f64;
    let tick_label_distance = 20_f64;
    let inner_height = format!("{}", inner_height);
    let inner_width = format!("{:.0}", plot_size);
    let inner_view_box = format!("0 0 {} {}", max_range_meter, max_alt_meter);
    let transform = format!(
        "translate({} {}) scale(1 -{}) translate(0 -{})",
        margin_size, margin_size, aspect_ratio, inner_height
    );
    let outer_size = plot_size + margin_size * 2_f64;
    let outer_view_box = format!("0 0 {:.0} {:.0}", outer_size, outer_size);
    let axis1 = PlotAxisConfig::new(
        PlotAxisName::X,
        0,
        max_range_meter as u32,
        (distance_axis.major_step * 1_000_f64) as u32,
        (distance_axis.minor_step * 1_000_f64) as u32,
    );
    let axis2 = PlotAxisConfig::new(PlotAxisName::Y, 0, max_alt_meter as u32, 5_000, 1_000);
    let grid_lines = create_grid_lines(&axis1, &axis2);

    let label_plotter = LabelPlotter::from_frame((&axis1, &axis2), plot_size, margin_size);
    let x_axis_label = label_plotter.plot(
        "Distance (km)",
        &LabelLoc::BottomAxis((axis1.end - axis1.start) as f64 / 2_f64),
        axis_label_distance,
        "x-axis",
    );
    let y_axis_label = label_plotter.plot(
        "Altitude (km)",
        &LabelLoc::LeftAxis((axis2.end - axis2.start) as f64 / 2_f64),
        axis_label_distance,
        "y-axis",
    );
    let tick_labels_axis1 = create_tick_labels(&label_plotter, &axis1, &tick_label_distance);
    let tick_labels_axis2 = create_tick_labels(&label_plotter, &axis2, &tick_label_distance);
    let el_labels = create_beam_labels(&label_plotter, el_labels, &tick_label_distance);

    html! {
        <svg id="viewer" viewBox={ outer_view_box }>
            <g transform={ transform }>
                <svg viewBox={ inner_view_box } width={ inner_width } height={ inner_height }>
                    { grid_lines }
                    { polylines }
                    <rect width="100%" height="100%" class="frame" />
                </svg>
            </g>
            { x_axis_label }
            { y_axis_label }
            { tick_labels_axis1 }
            { tick_labels_axis2 }
            { el_labels }
        </svg>
    }
}

fn create_polyline_for_beam<'a>(
    points: impl Iterator<Item = &'a calculator::AtmosphericPoint>,
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

enum BeamLabelLoc {
    MaxAltitude(f64),
    MaxDistance(f64),
}

fn locate_beam_labels<'a>(
    points: impl Iterator<Item = &'a calculator::AtmosphericPoint>,
    max_range_meter: &f64,
    max_alt_meter: &f64,
) -> BeamLabelLoc {
    let mut iter = points.peekable();
    while let Some(point) = iter.next() {
        if point.alt_meter > *max_alt_meter {
            return BeamLabelLoc::MaxAltitude(point.dist_meter);
        } else if point.dist_meter > *max_range_meter {
            return BeamLabelLoc::MaxDistance(point.alt_meter);
        } else if iter.peek().is_none() {
            return BeamLabelLoc::MaxDistance(point.alt_meter);
        }
    }
    unreachable!()
}

fn create_beam_labels<'a>(
    label_plotter: &LabelPlotter,
    labels: impl Iterator<Item = (&'a f64, BeamLabelLoc)>,
    tick_label_distance: &f64,
) -> Html {
    labels
        .map(|(el, loc)| {
            let label = format!("{:.0}°", el);
            let class = "elevation-label";
            match loc {
                BeamLabelLoc::MaxAltitude(dist) => label_plotter.plot(
                    &label,
                    &LabelLoc::TopAxis(dist),
                    *tick_label_distance,
                    class,
                ),
                BeamLabelLoc::MaxDistance(alt) => label_plotter.plot(
                    &label,
                    &LabelLoc::RightAxis(alt),
                    *tick_label_distance,
                    class,
                ),
            }
        })
        .collect::<Html>()
}

fn create_tick_labels(
    label_plotter: &LabelPlotter,
    axis: &PlotAxisConfig,
    tick_label_distance: &f64,
) -> Html {
    (axis.start..=axis.end)
        .step_by(axis.major_step as usize)
        .map(|val| {
            let label = format!("{:.0}", val / 1000); // km
            let loc = match axis.name {
                PlotAxisName::X => LabelLoc::BottomAxis(val as f64),
                PlotAxisName::Y => LabelLoc::LeftAxis(val as f64),
            };
            label_plotter.plot(&label, &loc, *tick_label_distance, "tick-label")
        })
        .collect::<Html>()
}

const DISTANCE_AXIS_OPTIONS: [DistanceAxis; 5] = [
    DistanceAxis::new(10, 2.0, 0.5),
    DistanceAxis::new(50, 10.0, 2.0),
    DistanceAxis::new(100, 20.0, 5.0),
    DistanceAxis::new(300, 50.0, 10.0),
    DistanceAxis::new(500, 100.0, 20.0),
];
const DEFAULT_OBS_DISTANCE_AXIS_OPTION_INDEX: usize = 3;

#[derive(PartialEq, Clone)]
pub struct DistanceAxis {
    // all in km
    max: u16,
    major_step: f64,
    minor_step: f64,
}

impl DistanceAxis {
    const fn new(max: u16, major_step: f64, minor_step: f64) -> Self {
        Self {
            max,
            major_step,
            minor_step,
        }
    }
}

fn create_html_select(id: String, current_index: usize, on_change: Callback<Event>) -> Html {
    let value = format!("{}", current_index);

    let options = DISTANCE_AXIS_OPTIONS
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let km = format!("{}", entry.max);
            let selected = index == current_index;
            let index = format!("{}", index);
            html! {<option value={ index } selected={ selected }>{ km }</option>}
        })
        .collect::<Html>();

    html! {
        <select onchange={ on_change }
            id={ id }
            value={ value }
        >
            { options }
        </select>
    }
}

#[function_component(App)]
fn app() -> Html {
    let alt_meter_handle = use_state(|| 0.0_f64);
    let alt_meter = *alt_meter_handle;
    let lat_deg_handle = use_state(|| 0.0_f64);
    let lat_deg = *lat_deg_handle;
    let distance_index_handle = use_state(|| DEFAULT_OBS_DISTANCE_AXIS_OPTION_INDEX);
    let distance_index = *distance_index_handle;

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

    let on_max_range_change = Callback::from(move |e: Event| {
        let target: EventTarget = e.target().expect("unknown event target");
        let value = target.unchecked_into::<HtmlInputElement>().value();
        if let Ok(value) = value.parse() {
            distance_index_handle.set(value);
        }
        // if value is not numeric, just ignore.
    });

    let alt_meter_value = format!("{}", alt_meter);
    let lat_deg_value = format!("{}", lat_deg);
    let select = create_html_select("max-range".to_owned(), distance_index, on_max_range_change);
    let distance_axis = DISTANCE_AXIS_OPTIONS[distance_index].clone();

    html! {
        <>
            <h1>{ "Radar beam path calculator" }</h1>
            <div id="main">
                <div id="side-menu">
                    <label for="radar-altitude">{"Site altitude (m)"}</label>
                    <input onchange={ on_alt_change }
                        id="radar-altitude"
                        type="number"
                        value={ alt_meter_value }
                    />
                    <br/>
                    <label for="radar-latitude">{"Site latitude (deg)"}</label>
                    <input onchange={ on_lat_change }
                        id="radar-latitude"
                        type="number"
                        value={ lat_deg_value }
                    />
                    <br/>
                    <label for="max-range">{"Max range (km)"}</label>
                    { select }
                </div>
                <div>
                    <BeamViewer lat_deg={ lat_deg } alt_meter={ alt_meter } distance_axis={ distance_axis } />
                </div>
            </div>
        </>
    }
}

fn main() {
    #[cfg(debug_assertions)]
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}

mod calculator;
mod plotter;
