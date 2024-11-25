//! This module provides the `solve` function to solve the following problem:
//!
//! Given a lowerbound (default 0) and an upperbound (defaunt +inf) and the std_dev of a normal distribution,
//! find out a center of the distribution, such that the mathmetical expectation of the distribution (after
//! truncated by the given lower and upper bound) equals an expected value that is between the lower and upper
//! bound.
//!
//! Enable `truncated-normal` feature to use this module.
//!
//! Use example can be found in the doc of `model::bw::NormalizedBwConfig::build_truncated`.
//!  

use statrs::function::erf::erf;
use std::f64::consts::PI;

fn integral(x: f64, t: f64, sigma: f64) -> f64 {
    let part1 = x * 0.5f64 * erf((t - x) / sigma / 2.0_f64.sqrt());
    let part2 = -sigma / (2.0f64 * PI).sqrt() * (-(t - x) * (t - x) * 0.5f64 / sigma / sigma).exp();

    part1 + part2
}

fn deri_integral(x: f64, t: f64, sigma: f64) -> f64 {
    let part1 = 0.5f64 * erf((t - x) / sigma / 2.0_f64.sqrt());
    let part2 =
        (-(t - x) * (t - x) * 0.5f64 / sigma / sigma).exp() * (-t) / (2.0f64 * PI).sqrt() / sigma;
    part1 + part2
}

fn cdf(t: f64, x: f64, sigma: f64) -> f64 {
    0.5f64 * (1f64 + erf((t - x) / sigma / 2f64.sqrt()))
}

fn deri_cdf(t: f64, x: f64, sigma: f64) -> f64 {
    -(-(t - x) * (t - x) / 2.0f64 / sigma / sigma).exp() / sigma / (2.0f64 * PI).sqrt()
}

fn truncated_band_width(avg: f64, sigma: f64, lower: Option<f64>, upper: Option<f64>) -> f64 {
    let upper_integral = if let Some(upper) = upper {
        integral(avg, upper, sigma)
    } else {
        //default upper as +inf
        avg * 0.5f64
    };

    let lower_integral = if let Some(lower) = lower {
        integral(avg, lower, sigma)
    } else {
        integral(avg, 0f64, sigma)
    };

    let upper_truncate = if let Some(upper) = upper {
        upper * (1f64 - cdf(upper, avg, sigma))
    } else {
        0.0f64
    };

    let lower_truncate = if let Some(lower) = lower {
        lower * cdf(lower, avg, sigma)
    } else {
        0.0f64
    };

    upper_integral - lower_integral + lower_truncate + upper_truncate
}

fn derivation_truncated_band_width(
    avg: f64,
    sigma: f64,
    lower: Option<f64>,
    upper: Option<f64>,
) -> f64 {
    let upper_integral = if let Some(upper) = upper {
        deri_integral(avg, upper, sigma)
    } else {
        //default upper as +inf
        0.5f64
    };

    let lower_integral = if let Some(lower) = lower {
        deri_integral(avg, lower, sigma)
    } else {
        deri_integral(avg, 0f64, sigma)
    };

    let upper_truncate = if let Some(upper) = upper {
        upper * (-deri_cdf(upper, avg, sigma))
    } else {
        0.0f64
    };

    let lower_truncate = if let Some(lower) = lower {
        lower * deri_cdf(lower, avg, sigma)
    } else {
        0.0f64
    };

    upper_integral - lower_integral + lower_truncate + upper_truncate
}

pub fn solve(x: f64, sigma: f64, mut lower: Option<f64>, upper: Option<f64>) -> Option<f64> {
    if sigma.abs() <= f64::EPSILON {
        return Some(x);
    }
    //sanity check
    if lower.is_some_and(|lower| lower >= x * (1.0 + f64::EPSILON)) {
        return lower;
    }

    if lower.is_none() && x <= f64::EPSILON {
        return 0f64.into();
    }

    if upper.is_some_and(|upper| upper * (1.0 + f64::EPSILON) <= x) {
        return upper;
    }

    let mut result = x;

    if lower.is_some_and(|l| l < 0.0) || lower.is_none() {
        lower = Some(0.0f64);
    }

    let mut last_diff = f64::MAX;
    let mut run_cnt = 10;

    while run_cnt > 0 {
        let f_x = truncated_band_width(result, sigma, lower, upper);

        let diff = (f_x - x).abs();
        if diff < last_diff {
            last_diff = diff;
            run_cnt = 100;
        } else {
            run_cnt -= 1;
        }

        result = result - (f_x - x) / derivation_truncated_band_width(result, sigma, lower, upper);
    }

    Some(result)
}
