use yew::prelude::*;

pub(crate) struct PlotAxisConfig {
    pub(crate) name: PlotAxisName,
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) major_step: u32,
    pub(crate) minor_step: u32,
}

impl PlotAxisConfig {
    pub(crate) fn new(
        name: PlotAxisName,
        start: u32,
        end: u32,
        major_step: u32,
        minor_step: u32,
    ) -> Self {
        Self {
            name,
            start,
            end,
            major_step,
            minor_step,
        }
    }
}

pub(crate) enum PlotAxisName {
    X,
    Y,
}

pub(crate) enum GridLineKind {
    Major,
    Minor,
}

pub(crate) fn create_grid_lines(axis1: &PlotAxisConfig, axis2: &PlotAxisConfig) -> Html {
    vec![
        create_grid_line(axis1, axis2, GridLineKind::Minor),
        create_grid_line(axis2, axis1, GridLineKind::Minor),
        create_grid_line(axis1, axis2, GridLineKind::Major),
        create_grid_line(axis2, axis1, GridLineKind::Major),
    ]
    .into_iter()
    .collect::<Html>()
}

fn create_grid_line(
    primary_axis: &PlotAxisConfig,
    secondary_axis: &PlotAxisConfig,
    kind: GridLineKind,
) -> Html {
    let (step, class_name) = match kind {
        GridLineKind::Major => (primary_axis.major_step, "major-grid-line"),
        GridLineKind::Minor => (primary_axis.minor_step, "minor-grid-line"),
    };
    let step = step as usize;
    let start = format!("{:.0}", secondary_axis.start);
    let end = format!("{:.0}", secondary_axis.end);

    (primary_axis.start..=primary_axis.end)
        .step_by(step)
        .map(|val| {
            let val = format!("{:.0}", val);
            match primary_axis.name {
                PlotAxisName::X => html! {
                    <line x1={ val.clone() } x2={ val.clone() } y1={ start.clone() } y2={ end.clone() } class={ class_name } />
                },
                PlotAxisName::Y => html! {
                    <line y1={ val.clone() } y2={ val.clone() } x1={ start.clone() } x2={ end.clone() } class={ class_name } />
                },
            }
        })
        .collect::<Html>()
}

pub(crate) struct LabelPlotter {
    origin: (f64, f64),
    end: (f64, f64),
    factor: (f64, f64),
}

impl LabelPlotter {
    fn new(origin: (f64, f64), end: (f64, f64), factor: (f64, f64)) -> Self {
        Self {
            origin,
            end,
            factor,
        }
    }

    pub(crate) fn from_frame(
        axis: (&PlotAxisConfig, &PlotAxisConfig),
        plot_size: f64,
        margin_size: f64,
    ) -> Self {
        let origin = (margin_size, margin_size + plot_size);
        let end = (margin_size + plot_size, margin_size);
        let factor = (
            plot_size / (axis.0.end - axis.0.start) as f64,
            plot_size / (axis.1.end - axis.1.start) as f64,
        );
        Self {
            origin,
            end,
            factor,
        }
    }

    pub(crate) fn plot(&self, label: &str, loc: &LabelLoc, distance: f64, class: &str) -> Html {
        let distance = match loc {
            LabelLoc::TopAxis(_) | LabelLoc::LeftAxis(_) => -distance,
            LabelLoc::RightAxis(_) | LabelLoc::BottomAxis(_) => distance,
        };
        let x = match loc {
            LabelLoc::TopAxis(x) | LabelLoc::BottomAxis(x) => self.origin.0 + x * self.factor.0,
            LabelLoc::RightAxis(_) => self.end.0 + distance,
            LabelLoc::LeftAxis(_) => self.origin.0 + distance,
        };
        let y = match loc {
            LabelLoc::TopAxis(_) => self.end.1 + distance,
            LabelLoc::RightAxis(y) | LabelLoc::LeftAxis(y) => self.origin.1 - y * self.factor.1,
            LabelLoc::BottomAxis(_) => self.origin.1 + distance,
        };

        let axis_class = match loc {
            LabelLoc::TopAxis(_) => "frame-top",
            LabelLoc::RightAxis(_) => "frame-right",
            LabelLoc::BottomAxis(_) => "frame-bottom",
            LabelLoc::LeftAxis(_) => "frame-left",
        };
        let class = classes!(class.to_owned(), axis_class);
        let x = format!("{:.0}", x);
        let y = format!("{:.0}", y);
        html! {
            <text
                class={ class }
                x={ x }
                y={ y }
            >
                { label }
            </text>
        }
    }
}

pub(crate) enum LabelLoc {
    TopAxis(f64),
    RightAxis(f64),
    BottomAxis(f64),
    LeftAxis(f64),
}
