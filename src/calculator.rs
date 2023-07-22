use std::{iter::Peekable, ops::Range, slice::Iter};

#[derive(Debug)]
pub(crate) struct AtmosphericPoint {
    pub(crate) alt_meter: f64,
    pub(crate) dist_meter: f64,
}

pub(crate) const ELEVATION_FACTOR: f64 = 0.1;

pub(crate) struct ElevationRange {
    pub(crate) start: u16,
    pub(crate) step: u16,
}

impl ElevationRange {
    pub(crate) fn new(start: u16, step: u16) -> Self {
        Self { start, step }
    }
}

pub(crate) fn iter_elevations(ranges: &[ElevationRange]) -> impl Iterator<Item = f64> + '_ {
    let range_iter = ElevationRangeIterator::new(ranges);
    range_iter
        .flat_map(|(r, step)| r.into_iter().step_by(step.into()))
        .map(|i| i as f64 * ELEVATION_FACTOR)
}

pub(crate) struct ElevationRangeIterator<'r>(Peekable<Iter<'r, ElevationRange>>);

impl<'r> ElevationRangeIterator<'r> {
    pub(crate) fn new(ranges: &'r [ElevationRange]) -> Self {
        Self(ranges.iter().peekable())
    }
}

impl<'r> Iterator for ElevationRangeIterator<'r> {
    type Item = (Range<u16>, u16);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.next();
        let end = self.0.peek().map(|r| r.start);
        next.map(|r| {
            if let Some(end) = end {
                (r.start..end, r.step)
            } else {
                (r.start..(r.start + 1), 1)
            }
        })
    }
}

pub(crate) fn calc_beam_points<'a>(
    max_range_meter: &f64,
    n_range_section: &'a i32,
    el_deg: &'a f64,
    lat_deg: &'a f64,
    alt_meter: &'a f64,
) -> impl Iterator<Item = AtmosphericPoint> + 'a {
    let div = max_range_meter / *n_range_section as f64;

    (0..=*n_range_section).map(move |i| {
        let r = div * i as f64;
        calc_altitude_and_distance_on_sphere(r, *el_deg, *lat_deg, *alt_meter)
    })
}

// References:
//
// - https://docs.wradlib.org/en/stable/generated/wradlib.georef.misc.site_distance.html
// - https://docs.wradlib.org/en/stable/generated/wradlib.georef.misc.bin_altitude.html
pub(crate) fn calc_altitude_and_distance_on_sphere(
    r_meter: f64,
    el_deg: f64,
    lat_deg: f64,
    alt_meter: f64,
) -> AtmosphericPoint {
    let el = el_deg.to_radians();
    let r_earth = calc_earth_radius(lat_deg);
    let r_eff = r_earth * 4_f64 / 3_f64;
    let sr = r_eff + alt_meter;
    let z = (r_meter * r_meter + sr * sr + r_meter * sr * 2_f64 * el.sin()).sqrt() - r_eff;
    let s = r_eff * ((r_meter * el.cos()) / (r_eff + z)).asin();
    AtmosphericPoint {
        alt_meter: z,
        dist_meter: s,
    }
}

pub(crate) const WGS84_RADIUS_EARTH_MAJOR: f64 = 6_378_137_f64;

pub(crate) const WGS84_INV_FLATTENING: f64 = 298.257_223_563;

pub(crate) fn calc_earth_radius(lat_deg: f64) -> f64 {
    let wgs84_radius_earth_minor =
        WGS84_RADIUS_EARTH_MAJOR * (1_f64 - 1_f64 / WGS84_INV_FLATTENING);
    let lat = lat_deg.to_radians();
    let cos_lat_2 = lat.cos().powf(2.into());
    let sin_lat_2 = lat.sin().powf(2.into());
    ((WGS84_RADIUS_EARTH_MAJOR.powf(4.into()) * cos_lat_2
        + wgs84_radius_earth_minor.powf(4.into()) * sin_lat_2)
        / (WGS84_RADIUS_EARTH_MAJOR.powf(2.into()) * cos_lat_2
            + wgs84_radius_earth_minor.powf(2.into()) * sin_lat_2))
        .sqrt()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn elevation_iteration() {
        let el_ranges = vec![
            ElevationRange::new(0, 5),
            ElevationRange::new(50, 10),
            ElevationRange::new(100, 20),
            ElevationRange::new(200, 50),
            ElevationRange::new(400, 0),
        ];
        let actual = iter_elevations(&el_ranges).collect::<Vec<_>>();
        let expected = vec![
            0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0,
            14.0, 16.0, 18.0, 20.0, 25.0, 30.0, 35.0, 40.0,
        ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn earth_radius_major() {
        assert_eq!(calc_earth_radius(0_f64), WGS84_RADIUS_EARTH_MAJOR);
    }

    #[test]
    fn earth_radius_minor() {
        assert_eq!(
            calc_earth_radius(90_f64),
            WGS84_RADIUS_EARTH_MAJOR * (1_f64 - 1_f64 / WGS84_INV_FLATTENING)
        );
    }
}
