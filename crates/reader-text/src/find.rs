//! Complete local-find ranges with bounded memory and indexed disk spill.
//!
//! Uses the fixed-record spool mechanism from little-screen-clip's text search.
//! GUI hosts request only matches intersecting their bounded visible fragments.

use regex::RegexBuilder;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::ops::Range;

const RECORD_BYTES: u64 = 16;
const MEMORY_LIMIT: usize = 1024 * 1024;

/// Literal or regular-expression search settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FindOptions {
    /// Exact query text, without source normalization.
    pub query: String,
    /// Whether the query is a regular expression.
    pub regex: bool,
    /// Whether case must match exactly.
    pub case_sensitive: bool,
}

/// Indexed match coordinates; memory stays bounded even for dense source matches.
pub struct MatchSpool {
    file: tempfile::SpooledTempFile,
    count: u64,
}

impl MatchSpool {
    /// Finds all matches, periodically checking whether its owning request retired.
    ///
    /// # Errors
    /// Returns regex, temporary-storage, or cancellation errors.
    pub fn build(
        source: &str,
        options: &FindOptions,
        cancelled: impl Fn() -> bool,
    ) -> Result<Self, String> {
        let mut spool = Self {
            file: tempfile::spooled_tempfile(MEMORY_LIMIT),
            count: 0,
        };
        if options.query.is_empty() {
            return Ok(spool);
        }
        let pattern = if options.regex {
            options.query.clone()
        } else {
            regex::escape(&options.query)
        };
        let regex = RegexBuilder::new(&pattern)
            .case_insensitive(!options.case_sensitive)
            .build()
            .map_err(|error| error.to_string())?;
        for (index, matched) in regex.find_iter(source).enumerate() {
            if index % 256 == 0 && cancelled() {
                return Err("Search cancelled".into());
            }
            spool
                .append(matched.start(), matched.end())
                .map_err(|error| error.to_string())?;
        }
        Ok(spool)
    }

    fn append(&mut self, start: usize, end: usize) -> io::Result<()> {
        self.file.write_all(&(start as u64).to_le_bytes())?;
        self.file.write_all(&(end as u64).to_le_bytes())?;
        self.count += 1;
        Ok(())
    }

    /// Returns the complete match count, including ranges spilled to disk.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.count
    }

    /// Returns whether the source has no matches.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Reads one exact match without materializing the entire result list.
    ///
    /// # Errors
    /// Returns an error for unavailable storage or an invalid index/platform offset.
    pub fn get(&mut self, index: u64) -> io::Result<Range<usize>> {
        if index >= self.count {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Match index is out of bounds",
            ));
        }
        self.file.seek(SeekFrom::Start(index * RECORD_BYTES))?;
        let mut record = [0_u8; 16];
        self.file.read_exact(&mut record)?;
        let start = usize::try_from(u64::from_le_bytes(
            record[..8].try_into().expect("eight-byte start"),
        ))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        let end = usize::try_from(u64::from_le_bytes(
            record[8..].try_into().expect("eight-byte end"),
        ))
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        Ok(start..end)
    }

    /// Reads only matches intersecting a bounded source fragment.
    ///
    /// # Errors
    /// Returns a storage error if the spool cannot be read.
    pub fn in_fragment(&mut self, fragment: Range<usize>) -> io::Result<Vec<(u64, Range<usize>)>> {
        let (mut low, mut high) = (0, self.count);
        while low < high {
            let mid = low + (high - low) / 2;
            let range = self.get(mid)?;
            if range.end < fragment.start {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        let mut matches = Vec::new();
        while low < self.count {
            let range = self.get(low)?;
            if range.start > fragment.end {
                break;
            }
            let overlaps = if range.is_empty() {
                range.start >= fragment.start
            } else {
                range.start < fragment.end && range.end > fragment.start
            };
            if overlaps {
                matches.push((low, range));
            }
            low += 1;
        }
        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_matches_spill_without_losing_count_or_coordinates() {
        let text = "a".repeat(100_000);
        let options = FindOptions {
            query: "a".into(),
            ..Default::default()
        };
        let mut spool = MatchSpool::build(&text, &options, || false).unwrap();
        assert_eq!(spool.len(), 100_000);
        assert!(spool.file.is_rolled());
        assert_eq!(spool.get(99_999).unwrap(), 99_999..100_000);
        assert_eq!(
            spool.in_fragment(99_997..99_999).unwrap(),
            vec![(99_997, 99_997..99_998), (99_998, 99_998..99_999)]
        );
        assert!(spool.get(100_000).is_err());
    }

    #[test]
    fn zero_width_matches_include_empty_lines_and_end_of_file() {
        let mut spool = MatchSpool::build(
            "one\n\nlast",
            &FindOptions {
                query: "(?m)$".into(),
                regex: true,
                ..Default::default()
            },
            || false,
        )
        .unwrap();
        assert_eq!(spool.in_fragment(0..3).unwrap(), vec![(0, 3..3)]);
        assert_eq!(spool.in_fragment(4..4).unwrap(), vec![(1, 4..4)]);
        assert_eq!(spool.in_fragment(5..9).unwrap(), vec![(2, 9..9)]);
    }

    #[test]
    fn unicode_literals_regex_errors_and_cancellation_are_distinct() {
        let mut spool = MatchSpool::build(
            "é😀 É😀",
            &FindOptions {
                query: "é😀".into(),
                ..Default::default()
            },
            || false,
        )
        .unwrap();
        assert_eq!(spool.len(), 2);
        assert_eq!(spool.get(1).unwrap(), 7..13);
        assert!(MatchSpool::build(
            "data",
            &FindOptions {
                query: "[".into(),
                regex: true,
                ..Default::default()
            },
            || false
        )
        .is_err());
        assert!(MatchSpool::build(
            "data",
            &FindOptions {
                query: "data".into(),
                ..Default::default()
            },
            || true
        )
        .is_err());
    }
}
