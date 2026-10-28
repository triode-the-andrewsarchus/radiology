//! Incremental variable-height source geometry.
//!
//! Extracted from little-screen-clip's text preview row-height mechanism, keeping
//! the bounded block Fenwick representation without application dependencies.

const BLOCK_SIZE: usize = 32;

/// Cumulative row heights, updated through a block Fenwick tree as wrapping is
/// measured. Unmeasured rows start at one font row; only heights are retained.
#[derive(Default)]
pub struct RowHeights {
    key: Option<(f32, f32, f32)>,
    heights: Vec<f32>,
    /// Fenwick sums over fixed-size blocks, not individual rows.
    sums: Vec<f64>,
    /// Next row awaiting offscreen measurement by the host.
    pub next_measurement: usize,
}

impl RowHeights {
    /// Invalidates estimates when geometry or row count changes.
    pub fn reset(&mut self, count: usize, width: f32, height: f32, pixels_per_point: f32) {
        let key = (width, height, pixels_per_point);
        if self.key == Some(key) && self.heights.len() == count {
            return;
        }
        self.key = Some(key);
        self.heights = vec![height; count];
        let block_count = count.div_ceil(BLOCK_SIZE);
        self.sums = vec![0.0; block_count + 1];
        for block in 0..block_count {
            let start = block * BLOCK_SIZE;
            let end = (start + BLOCK_SIZE).min(count);
            self.sums[block + 1] = (end - start) as f64 * height as f64;
        }
        for index in 1..=block_count {
            let parent = index + (index & index.wrapping_neg());
            if parent <= block_count {
                self.sums[parent] += self.sums[index];
            }
        }
        self.next_measurement = 0;
    }

    /// Updates one measured row without rescanning the document.
    pub fn set_height(&mut self, row: usize, height: f32) {
        let delta = height as f64 - self.heights[row] as f64;
        self.heights[row] = height;
        let mut index = row / BLOCK_SIZE + 1;
        while index < self.sums.len() {
            self.sums[index] += delta;
            index += index & index.wrapping_neg();
        }
    }

    /// Returns the cumulative height before a row.
    pub fn offset(&self, row: usize) -> f32 {
        let row = row.min(self.heights.len());
        let mut block = row / BLOCK_SIZE;
        let mut sum = 0.0_f64;
        while block > 0 {
            sum += self.sums[block];
            block &= block - 1;
        }
        let block_start = (row / BLOCK_SIZE) * BLOCK_SIZE;
        for height in &self.heights[block_start..row] {
            sum += *height as f64;
        }
        sum as f32
    }

    /// Returns the total measured/estimated document height.
    pub fn total_height(&self) -> f32 {
        self.offset(self.heights.len())
    }

    /// Finds the row containing a vertical offset.
    pub fn row_at(&self, offset: f32) -> usize {
        let block_count = self.heights.len().div_ceil(BLOCK_SIZE);
        if block_count == 0 {
            return 0;
        }
        let mut block = 0;
        let mut sum = 0.0;
        let mut step = block_count.checked_next_power_of_two().unwrap_or(0);
        while step > 0 {
            let next = block + step;
            if next <= block_count && sum + self.sums[next] <= offset as f64 {
                block = next;
                sum += self.sums[next];
            }
            step >>= 1;
        }
        let start = (block * BLOCK_SIZE).min(self.heights.len());
        let end = (start + BLOCK_SIZE).min(self.heights.len());
        for row in start..end {
            if sum + self.heights[row] as f64 > offset as f64 {
                return row;
            }
            sum += self.heights[row] as f64;
        }
        self.heights.len().saturating_sub(1)
    }
}
