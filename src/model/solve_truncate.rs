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
//!
//! Notation:
//! ```txt
//!     cdf(t, avg, sigma) is the cumulative distribution function of a normal distribution,
//!         whose center is avg and standard derivation is sigma, with respect to t
//!
//!     pdf(t, avg, sigma) is the probablity density function of a normal distribution,
//!         whose center is avg and standard derivation is sigma, with respect to t
//! ```
use statrs::function::erf::erf;
use std::f64::consts::PI;

/// Calculates the mathmetical expectation of the following distribution of t,
/// whose Cumulative Distribution Function (CDF(t)) is:
/// ```text
///     CDF(t) = 0, if t < lower
///     CDF(t) = 1, if t > upper
///     CDF(t) = cdf(t, avg, sigma)
/// ```
///
/// if lower or upper are given as `None`, default values of 0.0 and +inf respectively are used.
///
/// The calculation is seperated into three addicative parts.
/// 1. $$\int_{\text{lower}}^{\text{upper}} t \times \text{pdf}(t, \text{avg}, \text{sigma} ) \ \text dt $$
///     
///     The indefinite integral of above is calculated in `integral`
///
/// 2. $$upper \times (1 - cdf(upper, avg, sigma))$$
///
/// 3. $$lower \times cdf(lower, avg, sigma)$$
///
///
fn truncated_bandwidth(avg: f64, sigma: f64, lower: Option<f64>, upper: Option<f64>) -> f64 {
    //upper_integral - lower_integral is the integral described in the doc, which is part 1 of the calculation.
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

    // part 2 of the calculation as described in the doc.
    let upper_truncate = if let Some(upper) = upper {
        upper * (1f64 - cdf(upper, avg, sigma))
    } else {
        0.0f64
    };

    // part 3 of the calculation as described in the doc.
    let lower_truncate = if let Some(lower) = lower {
        lower * cdf(lower, avg, sigma)
    } else {
        0.0f64
    };

    upper_integral - lower_integral + lower_truncate + upper_truncate
}

/// An indefinite integral:
///     $$\int t \times \text{pdf}(t, \text{avg}, \text{sigma} ) \ \text dt $$
///
/// Used in `truncated_bandwidth`.
///
fn integral(avg: f64, t: f64, sigma: f64) -> f64 {
    let part1 = avg * 0.5f64 * erf((t - avg) / sigma / 2.0_f64.sqrt());
    let part2 =
        -sigma / (2.0f64 * PI).sqrt() * (-(t - avg) * (t - avg) * 0.5f64 / sigma / sigma).exp();

    part1 + part2
}

/// The cumulative distribution function of a normal distribution,
///     whose center is avg and standard derivation is sigma, with respect to t
///
/// Used in `truncated_bandwidth`.
///
///
fn cdf(t: f64, avg: f64, sigma: f64) -> f64 {
    0.5f64 * (1f64 + erf((t - avg) / sigma / 2f64.sqrt()))
}

/// The derivative of `truncated_bandwidth` with respect to `avg`.
/// As `truncated_bandwidth` is calculated in addicative parts, here calculates the derivative of it in
/// addicative parts, part by part.
///  
fn deri_truncated_bandwidth(avg: f64, sigma: f64, lower: Option<f64>, upper: Option<f64>) -> f64 {
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

/// Patial derivative of the following respect to `avg`.
///     $$\int t \times \text{pdf}(t, \text{avg}, \text{sigma} ) \ \text dt $$
///
/// Used in `derivative_truncated_bandwidth`.
///
fn deri_integral(avg: f64, t: f64, sigma: f64) -> f64 {
    let part1 = 0.5f64 * erf((t - avg) / sigma / 2.0_f64.sqrt());
    let part2 = (-(t - avg) * (t - avg) * 0.5f64 / sigma / sigma).exp() * (-t)
        / (2.0f64 * PI).sqrt()
        / sigma;
    part1 + part2
}

/// Patial derivative of the following respect to `avg`.
///     cdf(t, avg, sigma)
///
/// Used in `derivative_truncated_bandwidth`.
///
fn deri_cdf(t: f64, avg: f64, sigma: f64) -> f64 {
    -(-(t - avg) * (t - avg) / 2.0f64 / sigma / sigma).exp() / sigma / (2.0f64 * PI).sqrt()
}

/// Solve the problem descirbed at the head of this file with Newtown's method, which requires f(x) and f'(x) to
/// solve f(x) = 0. Here f(x) is `truncated_bandwidth` and f'(x) us `derivative_truncated_bandwidth`
///
///
/// Parameters:
///     x : target mathematical expectation of the truncated normal distribution
///     sigma: the standard deviation of the normal distribution before truncation
///     lower: the lower bound of the truncation, defalut 0 if None is provided
///     upper: the upper bound of the truncation, defalut +inf if None is provided
///
/// Return value:
///     if a solution is found for the problem, returns the cernter of the normal distribution before truncation
///     else (aka the sanity check of the parameters failed), returns None.
///
/// The units of the parameters above should be consistant, which is the unit of the return value.
///
/// ## Examples
///
/// ```
/// use netem_trace::model::solve_truncate::solve;
/// let a = solve(8.0, 2.0, Some(4.0), Some(12.0)).unwrap();
/// assert!((a-8.0).abs() < 0.000001);
///
/// let a = solve(10.0, 4.0, Some(4.0), Some(12.0)).unwrap();
/// assert_eq!(a, 11.145871035156846);
///
/// let a = solve(10.0, 20.0, None, None).unwrap();
/// assert_eq!(a, 3.7609851997619734);
///
/// let a = solve(5.0, 18.0, None, None).unwrap();
/// assert_eq!(a, -4.888296757781897);
///
/// let a = solve(10.0, 20.0, Some(7.0), Some(15.0)).unwrap();
/// assert_eq!(a, 4.584705225916618);
///
/// let a = solve(10.0, 0.01, Some(7.0), Some(15.0)).unwrap();
/// assert_eq!(a, 10.0);
///
/// let a = solve(10.0, 0.01, None, Some(15.0)).unwrap();
/// assert_eq!(a, 10.0);
///
/// let a = solve(10.0, 0.01, None, None).unwrap();
/// assert_eq!(a, 10.0);
///
/// let a = solve(10.0, 0.01, Some(3.0), None).unwrap();
/// assert_eq!(a, 10.0);
/// ```
///     
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
        let f_x = truncated_bandwidth(result, sigma, lower, upper);

        let diff = (f_x - x).abs();
        if diff < last_diff {
            last_diff = diff;
            run_cnt = 100;
        } else {
            run_cnt -= 1;
        }

        result = result - (f_x - x) / deri_truncated_bandwidth(result, sigma, lower, upper);
    }

    Some(result)
}

#[cfg(test)]
mod tests {

    use super::*;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    fn test_deri<F, G>(func: F, deri: G, low: f64, high: f64)
    where
        F: Fn(f64) -> f64,
        G: Fn(f64) -> f64,
    {
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..1000 {
            let x = rng.gen_range(low..high);
            let eps = 5E-8 * (low + high);
            let delta1 = func(x + eps) - func(x);
            let delta2 = eps * deri(x + eps * 0.5);
            dbg!(delta1, delta2);
            if delta1 * delta2 > 0.0 {
                assert!(delta1 / delta2 < 1.0000001);
                assert!(delta2 / delta1 < 1.0000001);
            } else {
                assert!(delta1.abs() < f32::EPSILON.into());
                assert!(delta2.abs() < f32::EPSILON.into());
            }
        }
    }

    #[test]
    fn test_truncated_bandwidth() {
        assert_eq!(
            truncated_bandwidth(10.0, 5.0, None, None),
            10.042453513094314
        );

        test_deri(
            |x| truncated_bandwidth(x, 3.0, None, None),
            |x| deri_truncated_bandwidth(x, 3.0, None, None),
            0.0,
            10.0,
        );

        test_deri(
            |x| truncated_bandwidth(x, 3.0, Some(3.0), None),
            |x| deri_truncated_bandwidth(x, 3.0, Some(3.0), None),
            0.0,
            10.0,
        );

        test_deri(
            |x| truncated_bandwidth(x, 3.0, Some(3.0), Some(20.0)),
            |x| deri_truncated_bandwidth(x, 3.0, Some(3.0), Some(20.0)),
            0.0,
            10.0,
        );
    }

    #[test]
    fn test_integral() {
        assert_eq!(integral(10.0, 5.0, 2.0), -4.972959947732017);
        assert_eq!(integral(10.0, 1E9, 2.0), 5.0);
        assert_eq!(integral(10.0, -1E9, 2.0), -5.0);

        test_deri(
            |x| integral(x, 8.0, 5.0),
            |x| deri_integral(x, 8.0, 5.0),
            6.0,
            10.0,
        );

        test_deri(
            |x| integral(x, 8.0, 15.0),
            |x| deri_integral(x, 8.0, 15.0),
            6.0,
            10.0,
        );
    }

    #[test]
    fn test_cdf() {
        assert_eq!(
            cdf(12.0, 12.0, 4.0) * 2.0 - 1.0, // 0 sigma
            0.0
        );
        assert_eq!(
            cdf(16.0, 12.0, 4.0) - cdf(8.0, 12.0, 4.0), // 1 sigma
            0.6826894921098856
        );
        assert_eq!(
            cdf(20.0, 12.0, 4.0) - cdf(4.0, 12.0, 4.0), // 2 sigma
            0.9544997361056748
        );
        assert_eq!(
            cdf(24.0, 12.0, 4.0) - cdf(0.0, 12.0, 4.0), // 2 sigma
            0.997300203936851
        );

        test_deri(|x| cdf(8.0, x, 5.0), |x| deri_cdf(8.0, x, 5.0), 6.0, 10.0);

        test_deri(
            |x| cdf(123.0, x, 15.0),
            |x| deri_cdf(123.0, x, 15.0),
            110.0,
            136.0,
        );
    }
}
