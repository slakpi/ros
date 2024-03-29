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

  /// Construct a new RangeSet with a list of ranges.
  ///
  /// # Parameters
  ///
  /// * `ranges` - A list of ranges to insert into the new set.
  ///
  /// # Returns
  ///
  /// A new RangeSet.
  pub fn new_with_ranges(ranges: &[Range]) -> Self {
    let mut set = Self::new();

    for range in ranges {
      set.insert_range(*range);
    }

    set.trim_ranges();

    set
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

  /// Exclude a range from the set.
  ///
  /// # Parameters
  ///
  /// * `excl` - The range to exclude.
  /// * `align` - The alignment to use for the exclusion.
  pub fn exclude_range(&mut self, excl: &Range, align: usize) {
    if excl.size == 0 {
      return;
    }

    let mut i = 0usize;

    while i < self.count {
      let split = self.ranges[i].split_range(excl, align);
      let mut a_none = false;
      let mut b_none = false;

      // If the first element is valid, the current range can simply be
      // replaced.
      if let Some(a) = split.0 {
        self.ranges[i] = a;
      } else {
        a_none = true;
      }

      // If the second element is valid, but the first is not, simply replace
      // the current range. Otherwise, insert the new range after the current
      // range. If inserting, increment the index an extra time.
      if let Some(b) = split.1 {
        if a_none {
          self.ranges[i] = b;
        } else if self.count < SET_SIZE {
          self.ranges.copy_within(i..self.count, i + 1);
          self.ranges[i + 1] = b;
          self.count += 1;
          i += 1;
        } else {
          debug_assert!(false, "Could not split range; set is full.");
        }
      } else {
        b_none = true;
      }

      // If neither element is valid, remove the current range. Do not increment
      // the index yet.
      if a_none && b_none {
        self.ranges.copy_within((i + 1)..self.count, i);
        self.count -= 1;
        continue;
      }

      i += 1;
    }

    self.trim_empty_ranges();
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
