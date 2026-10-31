//! Horizontal measurements for bounded fragments sharing a projected source row.

use std::ops::Range;

#[derive(Debug)]
struct Row {
    fragments: Range<usize>,
    width: f64,
    measured_until: usize,
}

/// Groups bounded fragments into unwrapped rows with incremental width measurements.
///
/// Each fragment retains its own layout budget. Hosts place fragments from the
/// same row side by side, using estimates until their bounded layouts are ready.
/// Source coordinates and renderer/font ownership remain with the host.
///
/// # Examples
///
/// ```
/// use reader_text::UnwrappedRows;
///
/// let mut rows = UnwrappedRows::new([(true, 80.0), (false, 80.0), (true, 20.0)]);
/// assert_eq!(rows.row(1), 0..2);
/// assert_eq!(rows.offset(1), 80.0);
/// assert!(!rows.prefix_measured(1));
/// rows.set_width(1, 64.0);
/// rows.set_width(0, 72.0);
/// assert!(rows.prefix_measured(1));
/// assert_eq!(rows.offset(1), 72.0);
/// assert_eq!(rows.row_width(0), 136.0);
/// ```
#[derive(Debug, Default)]
pub struct UnwrappedRows {
    rows: Vec<Row>,
    row_indices: Vec<usize>,
    widths: Vec<f64>,
    offsets: Vec<f64>,
    measured: Vec<bool>,
}

impl UnwrappedRows {
    /// Builds rows from `(first_in_row, estimated_width)` fragment measurements.
    ///
    /// The first fragment starts a row even if its flag is false. A true flag
    /// starts every subsequent row. Zero-width fragments and empty input are valid.
    ///
    /// # Panics
    ///
    /// Panics if an estimate is negative or not finite.
    #[must_use]
    pub fn new(fragments: impl IntoIterator<Item = (bool, f32)>) -> Self {
        let mut result = Self::default();
        for (first, width) in fragments {
            assert!(width.is_finite() && width >= 0.0, "invalid fragment width");
            let index = result.widths.len();
            if first || result.rows.is_empty() {
                result.rows.push(Row {
                    fragments: index..index,
                    width: 0.0,
                    measured_until: index,
                });
            }
            let row_index = result.rows.len() - 1;
            let row = &mut result.rows[row_index];
            result.row_indices.push(row_index);
            result.widths.push(f64::from(width));
            result.offsets.push(row.width);
            result.measured.push(false);
            row.width += f64::from(width);
            row.fragments.end = index + 1;
        }
        result
    }

    /// Returns the fragment range sharing `fragment`'s projected visual row.
    ///
    /// # Panics
    ///
    /// Panics if `fragment` is outside the constructed fragments.
    #[must_use]
    pub fn row(&self, fragment: usize) -> Range<usize> {
        self.rows[self.row_indices[fragment]].fragments.clone()
    }

    /// Returns the horizontal offset within the fragment's row, in host units.
    ///
    /// # Panics
    ///
    /// Panics if `fragment` is outside the constructed fragments.
    #[must_use]
    pub fn offset(&self, fragment: usize) -> f64 {
        self.offsets[fragment]
    }

    /// Returns the measured or estimated width of one bounded fragment.
    ///
    /// # Panics
    ///
    /// Panics if `fragment` is outside the constructed fragments.
    #[must_use]
    pub fn width(&self, fragment: usize) -> f64 {
        self.widths[fragment]
    }

    /// Returns the total measured or estimated width of the fragment's row.
    ///
    /// # Panics
    ///
    /// Panics if `fragment` is outside the constructed fragments.
    #[must_use]
    pub fn row_width(&self, fragment: usize) -> f64 {
        self.rows[self.row_indices[fragment]].width
    }

    /// Returns the widest current row, or zero for an empty projection.
    #[must_use]
    pub fn max_width(&self) -> f64 {
        self.rows.iter().map(|row| row.width).fold(0.0, f64::max)
    }

    /// Returns the first unmeasured fragment in the row, or its exclusive end.
    ///
    /// Hosts can use this frontier to prioritize a destination's prefix without
    /// measuring unrelated rows or scanning the whole row for every frame.
    ///
    /// # Panics
    ///
    /// Panics if `fragment` is outside the constructed fragments.
    #[must_use]
    pub fn measured_until(&self, fragment: usize) -> usize {
        self.rows[self.row_indices[fragment]].measured_until
    }

    /// Returns whether this fragment and its whole horizontal prefix are measured.
    ///
    /// # Panics
    ///
    /// Panics if `fragment` is outside the constructed fragments.
    #[must_use]
    pub fn prefix_measured(&self, fragment: usize) -> bool {
        self.measured_until(fragment) > fragment
    }

    /// Records a shaped width, updating only the affected row's later offsets.
    ///
    /// Returns true when width or measurement coverage changes. Repeated identical
    /// measurements return false. Out-of-order measurements advance the frontier
    /// as soon as the preceding gaps are filled.
    ///
    /// # Panics
    ///
    /// Panics for an invalid fragment or a negative/non-finite width.
    pub fn set_width(&mut self, fragment: usize, width: f32) -> bool {
        assert!(width.is_finite() && width >= 0.0, "invalid fragment width");
        let row = &mut self.rows[self.row_indices[fragment]];
        let delta = f64::from(width) - self.widths[fragment];
        let changed = delta != 0.0 || !self.measured[fragment];
        self.widths[fragment] = f64::from(width);
        self.measured[fragment] = true;
        row.width += delta;
        for offset in &mut self.offsets[fragment + 1..row.fragments.end] {
            *offset += delta;
        }
        while row.measured_until < row.fragments.end && self.measured[row.measured_until] {
            row.measured_until += 1;
        }
        changed
    }

    /// Finds the fragment at `x`, clamping to the row's first or last fragment.
    ///
    /// The returned placement is provisional until its prefix is measured. Exact
    /// boundaries belong to the following fragment, including zero-width pieces.
    ///
    /// # Panics
    ///
    /// Panics for an invalid fragment or a non-finite coordinate.
    #[must_use]
    pub fn fragment_at_x(&self, row_fragment: usize, x: f64) -> usize {
        assert!(x.is_finite(), "invalid horizontal coordinate");
        let row = self.row(row_fragment);
        row.start
            + self.offsets[row.clone()]
                .partition_point(|offset| *offset <= x)
                .saturating_sub(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn out_of_order_widths_update_only_their_row_and_close_measurement_gaps() {
        let mut rows = UnwrappedRows::new([
            (true, 10.0),
            (false, 20.0),
            (false, 0.0),
            (true, 5.0),
            (false, 7.0),
        ]);
        assert_eq!(rows.row(2), 0..3);
        assert_eq!(rows.row(4), 3..5);
        assert_eq!(rows.max_width(), 30.0);
        rows.set_width(2, 3.0);
        assert_eq!(rows.measured_until(2), 0);
        rows.set_width(0, 4.0);
        assert_eq!(rows.measured_until(2), 1);
        rows.set_width(1, 5.0);
        assert!(rows.prefix_measured(2));
        assert_eq!(
            (rows.offset(1), rows.offset(2), rows.row_width(2)),
            (4.0, 9.0, 12.0)
        );
        assert_eq!((rows.offset(4), rows.row_width(4)), (5.0, 12.0));
        assert!(!rows.set_width(1, 5.0));
        assert_eq!(rows.fragment_at_x(0, -1.0), 0);
        assert_eq!(rows.fragment_at_x(0, 4.0), 1);
        assert_eq!(rows.fragment_at_x(0, 99.0), 2);
        assert_eq!(UnwrappedRows::default().max_width(), 0.0);
    }

    #[test]
    fn repeated_updates_match_linear_row_geometry() {
        let first: Vec<_> = (0..91).map(|index| index % 11 == 0).collect();
        let mut expected = vec![8.0_f64; first.len()];
        let mut rows = UnwrappedRows::new(first.iter().map(|first| (*first, 8.0)));
        for step in 0..257 {
            let index = (step * 37) % first.len();
            let width = (step % 29) as f32 / 4.0;
            expected[index] = f64::from(width);
            rows.set_width(index, width);
            for fragment in 0..first.len() {
                let range = rows.row(fragment);
                assert_eq!(
                    rows.offset(fragment),
                    expected[range.start..fragment].iter().sum::<f64>()
                );
                assert_eq!(
                    rows.row_width(fragment),
                    expected[range].iter().sum::<f64>()
                );
            }
        }
    }

    #[test]
    fn projected_unicode_and_fold_joined_fragments_keep_their_source_rows() {
        let text = format!(
            "é😀{} hidden\nmore hidden end\r\nnext",
            "value ".repeat(1300)
        );
        let start = text.find(" hidden").unwrap();
        let end = text.find(" end").unwrap();
        let hidden = start..end;
        let fragments = crate::projected_fragments(&text, std::slice::from_ref(&hidden), 4096);
        let rows = UnwrappedRows::new(
            fragments
                .iter()
                .map(|fragment| (fragment.first_in_line, fragment.visible_len() as f32)),
        );
        let last = rows.row(0).end - 1;
        assert_eq!(rows.row(last), 0..last + 1);
        assert_eq!(
            rows.offset(last),
            fragments[..last]
                .iter()
                .map(|part| part.visible_len() as f64)
                .sum::<f64>()
        );
        assert!(fragments
            .iter()
            .all(|fragment| fragment.visible_len() <= 4096));
        assert!(fragments[rows.row(0).end].first_in_line);
    }
}
