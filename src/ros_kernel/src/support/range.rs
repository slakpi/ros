//! Range data structure.

use crate::arch::bits;

/// Range ordering.
///
/// * `MutuallyExclusiveLess` - Two ranges are mutually exclusive and the LHS
///   range occurs before the RHS range.
/// * `MutuallyExclusiveGreater` - Two ranges are mutually exclusive and the LHS
///   range occurs after the RHS range.
/// * `Less` - The LHS range partially overlaps the beginning of the RHS range.
/// * `Greater` - The LHS range partially overlaps the end of the RHS range.
/// * `Equal` - The two ranges are exactly equal.
/// * `Contains` - The LHS range fully contains the RHS range.
/// * `ContainedBy` - The LHS range is fully contained by the RHS range.
pub enum RangeOrder {
  MutuallyExclusiveLess,
  MutuallyExclusiveGreater,
  Less,
  Greater,
  Equal,
  Contains,
  ContainedBy,
}

/// A contiguous range of values.
#[derive(Copy, Clone)]
pub struct Range {
  pub base: usize,
  pub size: usize,
}

impl Range {
  /// Compare two ranges.
  ///
  /// # Parameters
  ///
  /// * `other` - The range to compare against.
  ///
  /// # Returns
  ///
  /// A range ordering.
  pub fn cmp(&self, other: &Range) -> RangeOrder {
    let my_end = self.base + self.size;
    let their_end = other.base + other.size;

    if self.base == other.base && self.size == other.size {
      // self  |---------------|
      // other |---------------|
      return RangeOrder::Equal;
    } else if my_end <= other.base {
      // self  |-----|
      // other        |---------------|
      // Includes the case where other's base = self's end as the ends are not
      // inclusive.
      return RangeOrder::MutuallyExclusiveLess;
    } else if their_end <= self.base {
      // self                   |-----|
      // other |---------------|
      // Includes the case where self's base = other's end as the ends are not
      // inclusive.
      return RangeOrder::MutuallyExclusiveGreater;
    } else if self.base <= other.base && my_end >= their_end {
      // self  |---------------|
      // other  |-----|
      // Includes the case where the bases are exactly equal.
      return RangeOrder::Contains;
    } else if other.base <= self.base && their_end >= my_end {
      // self           |-----|
      // other |---------------|
      // Includes the case where the ends are exactly equal.
      return RangeOrder::ContainedBy;
    } else if self.base < other.base {
      // self  |-----|
      // other    |---------------|
      return RangeOrder::Less;
    } else {
      // self               |-----|
      // other |---------------|
      return RangeOrder::Greater;
    }
  }

  /// Splits a range using an exclusion range.
  ///
  /// # Parameters
  ///
  /// * `excl` - The range to exclude.
  /// * `align` - The alignment value to use.
  ///
  /// # Details
  ///
  /// * If the ranges are mutually exclusive, returns the original range as the
  ///   first element in the tuple and None for the second.
  ///
  /// * If the exclusion range fully encompasses the range, returns None for
  ///   both elements of the tuple.
  ///
  /// * If the down page-aligned base, EE, of the exclusion range is greater
  ///   than the range base, returns a new range in the first element of the
  ///   tuple with the original base and a new size calculated using EE as the
  ///   end. Otherwise, returns None in the first element of the tuple.
  ///
  ///   If the up page-aligned end, EB, of the exclusion range is less than the
  ///   range end, returns a new range in the second element of the tuple with
  ///   EB as the base a new size calculated using the original end. Otherwise,
  ///   returns None in the second element of the tuple.
  ///
  /// The last case handles the exclusion range being fully encompassed by the
  /// range as well as the exclusion range overlapping either end of the range
  /// and handles returning None if the overlap results in empty ranges.
  ///
  /// # Returns
  ///
  /// A tuple with the resulting range(s) of the split. See details.
  pub fn split_range(&self, excl: &Range, align: usize) -> (Option<Range>, Option<Range>) {
    let my_end = self.base + self.size;
    let excl_end = excl.base + excl.size;
    let order = self.cmp(excl);

    match order {
      // There is no overlap between this range and the exclusion range. Simply
      // return this range.
      RangeOrder::MutuallyExclusiveLess | RangeOrder::MutuallyExclusiveGreater => {
        return (Some(*self), None);
      }
      // This range is either exactly equal to or fully contained by the
      // exclusion range. This range is complete excluded.
      RangeOrder::Equal | RangeOrder::ContainedBy => {
        return (None, None);
      }
      _ => {}
    }

    let ee = bits::align_down(excl.base, align);
    let eb = bits::align_up(excl_end, align);

    // self |---------------|
    // excl              |-----|
    //      |------------|
    //            a

    // self    |---------------|
    // excl |-----|
    //            |------------|
    //                  b

    // self |---------------|
    // excl     |-----|
    //      |---|     |-----|
    //        a          b

    let a = if ee > self.base {
      Some(Range {
        base: self.base,
        size: ee - self.base,
      })
    } else {
      None
    };

    let b = if eb < my_end {
      Some(Range {
        base: eb,
        size: my_end - eb,
      })
    } else {
      None
    };

    (a, b)
  }
}
