//! Range data structure.

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
  /// A range ordering or Err if the ranges are invalid.
  pub fn cmp(&self, other: &Range) -> Result<RangeOrder, ()> {
    if self.size == 0 || other.size == 0 {
      return Err(());
    }

    let my_end = self.base + (self.size - 1);
    let their_end = other.base + (other.size - 1);

    if self.base == other.base && self.size == other.size {
      // self  |---------------|
      // other |---------------|
      return Ok(RangeOrder::Equal);
    } else if my_end < other.base {
      // self  |-----|
      // other        |---------------|
      // Includes the case where other's base = self's end as the ends are not
      // inclusive.
      return Ok(RangeOrder::MutuallyExclusiveLess);
    } else if their_end < self.base {
      // self                   |-----|
      // other |---------------|
      // Includes the case where self's base = other's end as the ends are not
      // inclusive.
      return Ok(RangeOrder::MutuallyExclusiveGreater);
    } else if self.base <= other.base && my_end >= their_end {
      // self  |---------------|
      // other  |-----|
      // Includes the case where the bases are exactly equal.
      return Ok(RangeOrder::Contains);
    } else if other.base <= self.base && their_end >= my_end {
      // self           |-----|
      // other |---------------|
      // Includes the case where the ends are exactly equal.
      return Ok(RangeOrder::ContainedBy);
    } else if self.base < other.base {
      // self  |-----|
      // other    |---------------|
      return Ok(RangeOrder::Less);
    } else {
      // self               |-----|
      // other |---------------|
      return Ok(RangeOrder::Greater);
    }
  }

  /// Splits a range using an exclusion range.
  ///
  /// # Parameters
  ///
  /// * `excl` - The range to exclude.
  ///
  /// # Details
  ///
  /// * If the ranges are mutually exclusive, returns the original range as the
  ///   first element in the tuple and None for the second.
  ///
  /// * If the exclusion range fully encompasses the range, returns None for
  ///   both elements of the tuple.
  ///
  /// * If the base of the exclusion range is greater than the range base,
  ///   returns a new range in the first element of the tuple with the original
  ///   base and a new size calculated using the exclusion range base as the
  ///   end. Otherwise, returns None in the first element of the tuple.
  ///
  ///   If the end of the exclusion range is less than the range end, returns a
  ///   new range in the second element of the tuple with the exclusion range
  ///   base as the base with a new size calculated using the original end.
  ///   Otherwise, returns None in the second element of the tuple.
  ///
  /// The last case handles the exclusion range being fully encompassed by the
  /// range as well as the exclusion range overlapping either end of the range
  /// and handles returning None if the overlap results in empty ranges.
  ///
  /// # Returns
  ///
  /// A tuple with the resulting range(s) of the split. See details.
  pub fn split_range(&self, excl: &Range) -> Result<(Option<Range>, Option<Range>), ()> {
    let order = self.cmp(excl)?;

    match order {
      // There is no overlap between this range and the exclusion range. Simply
      // return this range.
      RangeOrder::MutuallyExclusiveLess | RangeOrder::MutuallyExclusiveGreater => {
        return Ok((Some(*self), None));
      }

      // This range is either exactly equal to or fully contained by the
      // exclusion range. This range is complete excluded.
      RangeOrder::Equal | RangeOrder::ContainedBy => {
        return Ok((None, None));
      }

      _ => {}
    }

    let my_end = self.base + (self.size - 1);
    let excl_end = excl.base + (excl.size - 1);

    // The following two cases are mutually exclusive in the comparison result.
    //
    // self |---------------|
    // excl              |-----|
    //      |------------|
    //            a
    //
    // self    |---------------|
    // excl |-----|
    //            |------------|
    //                  b
    //
    // However, if the exclusion range is fully contained, the result is the
    // same as performing both of the above:
    //
    // self |---------------|
    // excl     |-----|
    //      |---|     |-----|
    //        a          b
    let a = match order {
      RangeOrder::Less | RangeOrder::Contains => Some(Range {
        base: self.base,
        size: excl.base - self.base,
      }),

      _ => None,
    };

    let b = match order {
      RangeOrder::Greater | RangeOrder::Contains => Some(Range {
        base: excl_end + 1,
        size: my_end - excl_end,
      }),

      _ => None,
    };

    Ok((a, b))
  }
}
