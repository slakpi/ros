//! Range data structure.

use super::bits;

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
}
