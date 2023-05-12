//! Range set data structure.

use super::range::{Range, RangeOrder};

/// Fixed-size, ordered set of Ranges.
pub struct RangeSet<const SET_SIZE: usize> {
  ranges: [Range; SET_SIZE],
  count: usize,
}

impl<const SET_SIZE: usize> RangeSet<SET_SIZE> {
  /// Construct a new RangeSet.
  ///
  /// # Returns
  ///
  /// An empty RangeSet.
  pub const fn new() -> Self {
    RangeSet {
      ranges: [Range { base: 0, size: 0 }; SET_SIZE],
      count: 0,
    }
  }

  /// Check if the set is empty.
  ///
  /// # Returns
  ///
  /// True if the set is empty, false otherwise.
  pub fn is_empty(&self) -> bool {
    self.count == 0
  }

  /// Get the length of the set.
  ///
  /// # Returns
  ///
  /// The number of ranges in the set.
  pub fn _len(&self) -> usize {
    self.count
  }

  /// Access the ranges.
  ///
  /// # Returns
  ///
  /// A slice with the valid ranges.
  pub fn get_ranges(&self) -> &[Range] {
    &self.ranges[0..self.count]
  }

  /// Insert a new range in to the set ordered by base. Ranges with the same
  /// base are ordered from first to last inserted.
  ///
  /// # Parameters
  ///
  /// * `range` - The new range to add to the set.
  pub fn insert_range(&mut self, range: Range) {
    if self.count >= SET_SIZE {
      return;
    }

    let mut ins = self.count;

    for i in 0..self.count {
      if range.base < self.ranges[i].base {
        ins = i;
        break;
      }
    }

    self.ranges.copy_within(ins..self.count, ins + 1);
    self.ranges[ins] = range;
    self.count += 1;
  }

  /// Combines ranges as necessary to ensure ranges do not overlap and removes
  /// any empty ranges.
  pub fn trim_ranges(&mut self) {
    self.trim_overlapping_ranges();
    self.trim_empty_ranges();
  }

  /// Removes empty ranges from the set.
  fn trim_empty_ranges(&mut self) {
    let mut i = 0usize;

    while i < self.count {
      if self.ranges[i].size > 0 {
        i += 1;
        continue;
      }

      self.ranges.copy_within((i + 1)..self.count, i);
      self.count -= 1;
    }
  }

  /// Removes overlapping ranges from the set.
  fn trim_overlapping_ranges(&mut self) {
    if self.count < 2 {
      return;
    }

    let mut i = 0usize;

    while i < self.count - 1 {
      match self.ranges[i].cmp(&self.ranges[i + 1]) {
        RangeOrder::Equal | RangeOrder::Contains => {
          // This range contains the next range, remove the next range.
          self.ranges.copy_within((i + 2)..self.count, i + 1);
        }
        RangeOrder::ContainedBy => {
          // The next range contains this range, remove this range.
          self.ranges.copy_within((i + 1)..self.count, i);
        }
        RangeOrder::Less | RangeOrder::Greater => {
          // This range overlaps the next. Union the ranges and remove the
          // extraneous range. Given that we know the ranges are sorted and
          // overlap exists, the unsigned math is safe.
          self.ranges[i].size =
            (self.ranges[i + 1].base + self.ranges[i + 1].size) - self.ranges[i].base;
          self.ranges.copy_within((i + 2)..self.count, i + 1);
        }
        // No overlap, move ahead.
        _ => i += 1,
      }
    }
  }
}
