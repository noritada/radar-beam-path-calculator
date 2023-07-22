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
