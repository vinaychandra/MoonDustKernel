use alloc::rc::Rc;
use core::ops::Bound::*;
use core::{
    cmp::{Ord, Ordering},
    fmt::Debug,
    ops::Bound,
};

pub fn low_bound_cmp<T: Ord>(a: &Bound<T>, b: &Bound<T>) -> Ordering {
    match (a, b) {
        (Included(low1), Included(low2)) => low1.cmp(low2),
        (Included(low1), Excluded(low2)) => {
            if low1 <= low2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(low1), Included(low2)) => {
            if low1 < low2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(low1), Excluded(low2)) => low1.cmp(low2),
        (Unbounded, Unbounded) => Ordering::Equal,
        (Unbounded, _) => Ordering::Less,
        (_, Unbounded) => Ordering::Greater,
    }
}

pub fn low_bound_min<T: Ord + Clone>(a: &Rc<Bound<T>>, b: &Rc<Bound<T>>) -> Rc<Bound<T>> {
    match low_bound_cmp(&*a, &*b) {
        Ordering::Less => a.clone(),
        _ => b.clone(),
    }
}

pub fn high_bound_cmp<T: Ord + Clone>(a: &Bound<T>, b: &Bound<T>) -> Ordering {
    match (a, b) {
        (Included(high1), Included(high2)) => high1.cmp(high2),
        (Included(high1), Excluded(high2)) => {
            if high1 < high2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(high1), Included(high2)) => {
            if high1 <= high2 {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (Excluded(high1), Excluded(high2)) => high1.cmp(high2),
        (Unbounded, Unbounded) => Ordering::Equal,
        (Unbounded, _) => Ordering::Greater,
        (_, Unbounded) => Ordering::Less,
    }
}

pub fn high_bound_max<T: Ord + Clone>(a: &Rc<Bound<T>>, b: &Rc<Bound<T>>) -> Rc<Bound<T>> {
    match high_bound_cmp(&*a, &*b) {
        Ordering::Less => b.clone(),
        _ => a.clone(),
    }
}

/// A data structure for representing intervals
#[derive(Debug, Clone, Hash)]
pub struct Interval<T: Ord + Clone> {
    pub(crate) low: Rc<Bound<T>>,
    pub(crate) high: Rc<Bound<T>>,
}

impl<T: Ord + Clone> Interval<T> {
    /// Construct a new Interval from two Bounds
    ///
    /// # Example
    /// ```
    /// # use im_interval_tree::Interval;
    /// use std::ops::Bound::*;
    /// let interval = Interval::new(Included(3), Excluded(5));
    /// ```
    pub fn new(low: Bound<T>, high: Bound<T>) -> Interval<T> {
        Interval {
            low: Rc::new(low),
            high: Rc::new(high),
        }
    }

    fn valid(interval: &Interval<T>) -> bool {
        match (&*interval.low, &*interval.high) {
            (Included(low), Included(high)) => low <= high,

            (Included(low), Excluded(high))
            | (Excluded(low), Included(high))
            | (Excluded(low), Excluded(high)) => low < high,

            _ => true,
        }
    }

    /// Get the overlap between two Intervals
    ///
    /// # Example
    /// ```
    /// # use im_interval_tree::Interval;
    /// # use std::ops::Bound::*;
    /// let interval = Interval::new(Included(1), Excluded(3));
    /// let overlaps = Interval::new(Included(2), Excluded(4));
    /// let no_overlap = Interval::new(Included(3), Excluded(4));
    ///
    /// assert_eq!(
    ///     interval.get_overlap(&overlaps),
    ///     Some(Interval::new(Included(2), Excluded(3)))
    /// );
    ///
    /// assert_eq!(interval.get_overlap(&no_overlap), None);
    /// ```
    pub fn get_overlap(&self, other: &Self) -> Option<Self> {
        let low = match low_bound_cmp(&*self.low, &*other.low) {
            Ordering::Less => other.low.clone(),
            _ => self.low.clone(),
        };
        let high = match high_bound_cmp(&*self.high, &*other.high) {
            Ordering::Less => self.high.clone(),
            _ => other.high.clone(),
        };
        let interval = Interval {
            low: low,
            high: high,
        };
        if Self::valid(&interval) {
            Some(interval)
        } else {
            None
        }
    }

    /// Check whether two intervals overlap
    ///
    /// # Example
    /// ```
    /// # use im_interval_tree::Interval;
    /// # use std::ops::Bound::*;
    /// let interval = Interval::new(Included(1), Excluded(3));
    /// let overlaps = Interval::new(Included(2), Excluded(4));
    /// let no_overlap = Interval::new(Included(3), Excluded(4));
    ///
    /// assert_eq!(interval.overlaps(&overlaps), true);
    /// assert_eq!(interval.overlaps(&no_overlap), false);
    /// ```
    pub fn overlaps(&self, other: &Self) -> bool {
        self.get_overlap(other).is_some()
    }

    /// Check whether an interval contains another
    ///
    /// # Example
    /// ```
    /// # use im_interval_tree::Interval;
    /// # use std::ops::Bound::*;
    /// let interval = Interval::new(Included(1), Excluded(4));
    /// let contained = Interval::new(Included(2), Excluded(3));
    /// let not_contained = Interval::new(Included(2), Excluded(6));
    ///
    /// assert_eq!(interval.contains(&contained), true);
    /// assert_eq!(interval.contains(&not_contained), false);
    /// ```
    pub fn contains(&self, other: &Self) -> bool {
        let left_side_lte = match low_bound_cmp(self.low(), other.low()) {
            Ordering::Greater => false,
            _ => true,
        };
        let right_side_gte = match high_bound_cmp(self.high(), other.high()) {
            Ordering::Less => false,
            _ => true,
        };
        left_side_lte && right_side_gte
    }

    /// Return the lower bound
    ///
    /// # Example
    /// ```
    /// # use im_interval_tree::Interval;
    /// # use std::ops::Bound::*;
    /// let interval = Interval::new(Included(3), Excluded(5));
    /// assert_eq!(interval.low(), &Included(3))
    /// ```
    pub fn low(&self) -> &Bound<T> {
        &*self.low
    }

    /// Return the upper bound
    ///
    /// # Example
    /// ```
    /// # use im_interval_tree::Interval;
    /// # use std::ops::Bound::*;
    /// let interval = Interval::new(Included(3), Excluded(5));
    /// assert_eq!(interval.high(), &Excluded(5))
    /// ```
    pub fn high(&self) -> &Bound<T> {
        &*self.high
    }
}

impl<T: Ord + Clone> PartialEq for Interval<T> {
    fn eq(&self, other: &Self) -> bool {
        self.low == other.low && self.high == other.high
    }
}

impl<T: Ord + Clone> Eq for Interval<T> {}

impl<T: Ord + Clone> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord + Clone> Ord for Interval<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        let low_bound_cmp = low_bound_cmp(&*self.low, &*other.low);
        if low_bound_cmp == Ordering::Equal {
            high_bound_cmp(&*self.high, &*other.high)
        } else {
            low_bound_cmp
        }
    }
}
