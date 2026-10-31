//! Protocol Buffers wire-format reader and SCIP wire schema models.

use thiserror::Error;

/// Describes errors encountered while reading Protocol Buffers wire format.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProtobufError {
    /// Data buffer ended prematurely.
    #[error("Truncated protobuf data")]
    Truncated,
    /// Encoded varint exceeded 64 bits.
    #[error("Malformed varint")]
    MalformedVarint,
    /// Wire type is unrecognized.
    #[error("Unsupported wire type: {0}")]
    UnsupportedWireType(u8),
    /// String payload was not valid UTF-8.
    #[error("Invalid UTF-8 string")]
    InvalidUtf8,
    /// The owner retired decoding or derived index construction.
    #[error("SCIP processing cancelled")]
    Cancelled,
}

pub(super) fn check_cancelled(cancelled: &(dyn Fn() -> bool + Sync)) -> Result<(), ProtobufError> {
    if cancelled() {
        Err(ProtobufError::Cancelled)
    } else {
        Ok(())
    }
}

/// Protobuf wire types supported by the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    /// Varint: int32, int64, uint32, bool, enum.
    Varint = 0,
    /// 64-bit: fixed64, sfixed64, double.
    Fixed64 = 1,
    /// Length-delimited: string, bytes, embedded messages, packed repeated fields.
    LengthDelimited = 2,
    /// 32-bit: fixed32, sfixed32, float.
    Fixed32 = 5,
}

impl WireType {
    /// Converts a raw 3-bit wire type integer into [`WireType`].
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError::UnsupportedWireType`] if `val` is not a known wire type.
    pub fn from_u8(val: u8) -> Result<Self, ProtobufError> {
        match val {
            0 => Ok(Self::Varint),
            1 => Ok(Self::Fixed64),
            2 => Ok(Self::LengthDelimited),
            5 => Ok(Self::Fixed32),
            other => Err(ProtobufError::UnsupportedWireType(other)),
        }
    }
}

/// A minimal zero-dependency Protocol Buffers reader for decoding SCIP indexes.
pub struct ProtobufReader<'a> {
    data: &'a [u8],
    offset: usize,
    cancelled: Option<&'a (dyn Fn() -> bool + Sync)>,
}

impl<'a> ProtobufReader<'a> {
    /// Creates a reader over the given byte slice.
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            offset: 0,
            cancelled: None,
        }
    }

    fn check(&self) -> Result<(), ProtobufError> {
        self.cancelled.map_or(Ok(()), check_cancelled)
    }

    fn message(&mut self) -> Result<Self, ProtobufError> {
        let data = self.read_length_delimited()?;
        Ok(Self {
            data,
            offset: 0,
            cancelled: self.cancelled,
        })
    }

    /// Returns `true` if all bytes have been consumed.
    #[must_use]
    pub fn is_at_end(&self) -> bool {
        self.offset >= self.data.len()
    }

    /// Reads the next field tag number and associated [`WireType`].
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if reading fails.
    pub fn read_tag(&mut self) -> Result<Option<(u32, WireType)>, ProtobufError> {
        self.check()?;
        if self.is_at_end() {
            return Ok(None);
        }
        let key = self.read_varint()?;
        let raw_wire = (key & 0x7) as u8;
        let wire = WireType::from_u8(raw_wire)?;
        let field = (key >> 3) as u32;
        Ok(Some((field, wire)))
    }

    /// Reads a variable-length integer.
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError::Truncated`] if the stream ends before the varint terminates.
    /// Returns [`ProtobufError::MalformedVarint`] if the encoded value exceeds 64 bits.
    pub fn read_varint(&mut self) -> Result<u64, ProtobufError> {
        self.check()?;
        let mut result = 0u64;
        let mut shift = 0u32;
        while self.offset < self.data.len() {
            let byte = self.data[self.offset];
            self.offset += 1;
            if shift == 63 && byte > 1 {
                return Err(ProtobufError::MalformedVarint);
            }
            result |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 64 {
                return Err(ProtobufError::MalformedVarint);
            }
        }
        Err(ProtobufError::Truncated)
    }

    /// Reads a 32-bit signed integer encoded as a varint.
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if reading the varint fails.
    pub fn read_int32(&mut self) -> Result<i32, ProtobufError> {
        Ok(self.read_varint()? as i32)
    }

    /// Reads a length-delimited byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError::Truncated`] if the slice length extends beyond available data.
    pub fn read_length_delimited(&mut self) -> Result<&'a [u8], ProtobufError> {
        let len = usize::try_from(self.read_varint()?).map_err(|_| ProtobufError::Truncated)?;
        if len > self.data.len().saturating_sub(self.offset) {
            return Err(ProtobufError::Truncated);
        }
        let slice = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Ok(slice)
    }

    /// Reads a length-delimited UTF-8 string.
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError::InvalidUtf8`] if bytes are not valid UTF-8.
    pub fn read_string(&mut self) -> Result<String, ProtobufError> {
        let bytes = self.read_length_delimited()?;
        self.check()?;
        let mut output = Vec::with_capacity(bytes.len());
        for chunk in bytes.chunks(64 * 1024) {
            self.check()?;
            output.extend_from_slice(chunk);
        }
        let text = String::from_utf8(output).map_err(|_| ProtobufError::InvalidUtf8);
        self.check()?;
        text
    }

    /// Reads packed repeated 32-bit integer values.
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if payload fails decoding.
    pub fn read_packed_int32(&mut self) -> Result<Vec<i32>, ProtobufError> {
        let mut reader = self.message()?;
        let mut out = Vec::new();
        while !reader.is_at_end() {
            out.push(reader.read_int32()?);
        }
        reader.check()?;
        Ok(out)
    }

    /// Reads repeated 32-bit integers, whether packed (length-delimited) or unpacked (varint).
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if payload fails decoding.
    pub fn read_repeated_int32(
        &mut self,
        wire: WireType,
        out: &mut Vec<i32>,
    ) -> Result<(), ProtobufError> {
        match wire {
            WireType::LengthDelimited => {
                let vals = self.read_packed_int32()?;
                out.extend(vals);
            }
            WireType::Varint => {
                out.push(self.read_int32()?);
            }
            other => self.skip_field(other)?,
        }
        Ok(())
    }

    /// Skips over an unhandled field matching `wire`.
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if the field extends past the end of data.
    pub fn skip_field(&mut self, wire: WireType) -> Result<(), ProtobufError> {
        self.check()?;
        match wire {
            WireType::Varint => {
                self.read_varint()?;
            }
            WireType::Fixed64 => {
                if self.offset + 8 > self.data.len() {
                    return Err(ProtobufError::Truncated);
                }
                self.offset += 8;
            }
            WireType::LengthDelimited => {
                self.read_length_delimited()?;
            }
            WireType::Fixed32 => {
                if self.offset + 4 > self.data.len() {
                    return Err(ProtobufError::Truncated);
                }
                self.offset += 4;
            }
        }
        Ok(())
    }
}

/// Decoded top-level SCIP index representation.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ScipIndex {
    /// Source-file byte encoding from metadata, independent of occurrence columns.
    pub text_document_encoding: i32,
    /// Documents contained in the index.
    pub documents: Vec<ScipDocument>,
    /// External package symbol metadata.
    pub external_symbols: Vec<ScipSymbolInformation>,
}

/// Decoded SCIP document with occurrences and symbols.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ScipDocument {
    /// Column encoding (1 UTF-8, 2 UTF-16, 3 UTF-32; zero is unspecified).
    pub position_encoding: i32,
    /// Repository-relative path of the indexed file.
    pub relative_path: String,
    /// Language identifier reported by the indexer.
    pub language: String,
    /// Occurrences found within the document.
    pub occurrences: Vec<ScipOccurrence>,
    /// Defined symbols within this document.
    pub symbols: Vec<ScipSymbolInformation>,
}

/// Half-open character coordinate range `[start_line:start_col, end_line:end_col)`.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct ScipRange {
    /// Zero-based starting line.
    pub start_line: i32,
    /// Zero-based starting character column.
    pub start_col: i32,
    /// Zero-based ending line.
    pub end_line: i32,
    /// Zero-based ending character column.
    pub end_col: i32,
}

impl ScipRange {
    /// Decodes 3/4 packed coordinates, rejecting negative or reversed positions.
    #[must_use]
    pub fn from_packed(vals: &[i32]) -> Option<Self> {
        let range = match vals {
            [line, start, end] => Self {
                start_line: *line,
                start_col: *start,
                end_line: *line,
                end_col: *end,
            },
            [line, start, end_line, end] => Self {
                start_line: *line,
                start_col: *start,
                end_line: *end_line,
                end_col: *end,
            },
            _ => return None,
        };
        (range.start_line >= 0
            && range.start_col >= 0
            && range.end_col >= 0
            && (range.start_line, range.start_col) <= (range.end_line, range.end_col))
            .then_some(range)
    }
}

/// Bitflag indicating a symbol definition occurrence.
pub const SCIP_ROLE_DEFINITION: i32 = 1 << 0;
/// Bitflag indicating an import occurrence.
pub const SCIP_ROLE_IMPORT: i32 = 1 << 1;
/// Bitflag indicating a write access occurrence.
pub const SCIP_ROLE_WRITE: i32 = 1 << 2;
/// Bitflag indicating a read access occurrence.
pub const SCIP_ROLE_READ: i32 = 1 << 3;

/// An occurrence of a symbol at a specific range in a document.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ScipOccurrence {
    /// Source coordinate range.
    pub range: Option<ScipRange>,
    /// SCIP symbol string.
    pub symbol: String,
    /// Symbol role bitflags.
    pub symbol_roles: i32,
    /// Enclosing declaration range if provided.
    pub enclosing_range: Option<ScipRange>,
}

impl ScipOccurrence {
    /// Returns `true` if this occurrence defines the symbol.
    #[must_use]
    pub fn is_definition(&self) -> bool {
        (self.symbol_roles & SCIP_ROLE_DEFINITION) != 0
    }
}

/// Describes a relationship between two symbols (e.g. implementation, reference).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScipRelationship {
    /// Related symbol string identifier.
    pub symbol: String,
    /// Whether this is a reference relationship.
    pub is_reference: bool,
    /// Whether this is an implementation relationship.
    pub is_implementation: bool,
    /// Whether this defines a type.
    pub is_type_definition: bool,
    /// Whether this is a definition relationship.
    pub is_definition: bool,
}

/// Metadata and documentation for a semantic symbol.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ScipSymbolInformation {
    /// Unique SCIP symbol string identifier.
    pub symbol: String,
    /// Documentation comments associated with the symbol.
    pub documentation: Vec<String>,
    /// Relationships to other symbols.
    pub relationships: Vec<ScipRelationship>,
    /// SCIP symbol kind integer code.
    pub kind: i32,
    /// Short display name for tooltips and outlines.
    pub display_name: String,
    /// Signature documentation if available.
    pub signature: String,
    /// Enclosing symbol identifier if available.
    pub enclosing_symbol: String,
}

impl ScipIndex {
    /// Decodes a [`ScipIndex`] from raw protobuf wire bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ProtobufError`] if decoding fails.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtobufError> {
        Self::decode_cancellable(bytes, || false)
    }

    pub(crate) fn decode_cancellable(
        bytes: &[u8],
        cancelled: impl Fn() -> bool + Sync,
    ) -> Result<Self, ProtobufError> {
        let mut reader = ProtobufReader {
            data: bytes,
            offset: 0,
            cancelled: Some(&cancelled),
        };
        let mut index = ScipIndex::default();

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::LengthDelimited) => {
                    index.text_document_encoding = Self::decode_encoding(reader.message()?)?;
                }
                (2, WireType::LengthDelimited) => {
                    let doc_bytes = reader.message()?;
                    index.documents.push(Self::decode_document(doc_bytes)?);
                }
                (3, WireType::LengthDelimited) => {
                    let sym_bytes = reader.message()?;
                    index
                        .external_symbols
                        .push(Self::decode_symbol_info(sym_bytes)?);
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        Ok(index)
    }

    fn decode_encoding(mut reader: ProtobufReader<'_>) -> Result<i32, ProtobufError> {
        let mut encoding = 0;
        while let Some((field, wire)) = reader.read_tag()? {
            if field == 4 && wire == WireType::Varint {
                encoding = reader.read_int32()?;
            } else {
                reader.skip_field(wire)?;
            }
        }
        Ok(encoding)
    }

    fn decode_document(mut reader: ProtobufReader<'_>) -> Result<ScipDocument, ProtobufError> {
        let mut doc = ScipDocument::default();

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::LengthDelimited) => {
                    doc.relative_path = reader.read_string()?;
                }
                (2, WireType::LengthDelimited) => {
                    let occ_bytes = reader.message()?;
                    doc.occurrences.push(Self::decode_occurrence(occ_bytes)?);
                }
                (3, WireType::LengthDelimited) => {
                    let sym_bytes = reader.message()?;
                    doc.symbols.push(Self::decode_symbol_info(sym_bytes)?);
                }
                (4, WireType::LengthDelimited) => {
                    doc.language = reader.read_string()?;
                }
                (6, WireType::Varint) => doc.position_encoding = reader.read_int32()?,
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        Ok(doc)
    }

    fn decode_occurrence(mut reader: ProtobufReader<'_>) -> Result<ScipOccurrence, ProtobufError> {
        let mut occ = ScipOccurrence::default();
        let mut packed_range: Vec<i32> = Vec::new();
        let mut packed_enclosing: Vec<i32> = Vec::new();
        let mut typed_range: Option<Option<ScipRange>> = None;
        let mut typed_enclosing: Option<Option<ScipRange>> = None;

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, _) => {
                    reader.read_repeated_int32(wire, &mut packed_range)?;
                }
                (2, WireType::LengthDelimited) => {
                    occ.symbol = reader.read_string()?;
                }
                (3, WireType::Varint) => {
                    occ.symbol_roles = reader.read_int32()?;
                }
                (7, _) => {
                    reader.read_repeated_int32(wire, &mut packed_enclosing)?;
                }
                (8, WireType::LengthDelimited) => {
                    let msg = reader.message()?;
                    typed_range = Some(Self::decode_single_line_range(msg)?);
                }
                (9, WireType::LengthDelimited) => {
                    let msg = reader.message()?;
                    typed_range = Some(Self::decode_multi_line_range(msg)?);
                }
                (10, WireType::LengthDelimited) => {
                    let msg = reader.message()?;
                    typed_enclosing = Some(Self::decode_single_line_range(msg)?);
                }
                (11, WireType::LengthDelimited) => {
                    let msg = reader.message()?;
                    typed_enclosing = Some(Self::decode_multi_line_range(msg)?);
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        // Typed ranges take precedence over the packed form.
        occ.range = typed_range.unwrap_or_else(|| ScipRange::from_packed(&packed_range));
        occ.enclosing_range =
            typed_enclosing.unwrap_or_else(|| ScipRange::from_packed(&packed_enclosing));

        Ok(occ)
    }

    fn decode_single_line_range(
        mut reader: ProtobufReader<'_>,
    ) -> Result<Option<ScipRange>, ProtobufError> {
        let mut line = 0;
        let mut start = 0;
        let mut end = 0;
        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::Varint) => line = reader.read_int32()?,
                (2, WireType::Varint) => start = reader.read_int32()?,
                (3, WireType::Varint) => end = reader.read_int32()?,
                (_, wire) => reader.skip_field(wire)?,
            }
        }
        Ok(ScipRange::from_packed(&[line, start, end]))
    }

    fn decode_multi_line_range(
        mut reader: ProtobufReader<'_>,
    ) -> Result<Option<ScipRange>, ProtobufError> {
        let mut start_line = 0;
        let mut start_col = 0;
        let mut end_line = 0;
        let mut end_col = 0;
        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::Varint) => start_line = reader.read_int32()?,
                (2, WireType::Varint) => start_col = reader.read_int32()?,
                (3, WireType::Varint) => end_line = reader.read_int32()?,
                (4, WireType::Varint) => end_col = reader.read_int32()?,
                (_, wire) => reader.skip_field(wire)?,
            }
        }
        Ok(ScipRange::from_packed(&[
            start_line, start_col, end_line, end_col,
        ]))
    }

    fn decode_symbol_info(
        mut reader: ProtobufReader<'_>,
    ) -> Result<ScipSymbolInformation, ProtobufError> {
        let mut sym = ScipSymbolInformation::default();

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::LengthDelimited) => {
                    sym.symbol = reader.read_string()?;
                }
                (3, WireType::LengthDelimited) => {
                    sym.documentation.push(reader.read_string()?);
                }
                (4, WireType::LengthDelimited) => {
                    let rel_bytes = reader.message()?;
                    sym.relationships
                        .push(Self::decode_relationship(rel_bytes)?);
                }
                (5, WireType::Varint) => {
                    sym.kind = reader.read_int32()?;
                }
                (6, WireType::LengthDelimited) => {
                    sym.display_name = reader.read_string()?;
                }
                (7, WireType::LengthDelimited) => {
                    // `Signature` (historically a `Document`): only `text` (field 5) is used.
                    let mut sig_reader = reader.message()?;
                    while let Some((sig_field, sig_wire)) = sig_reader.read_tag()? {
                        if sig_field == 5 && sig_wire == WireType::LengthDelimited {
                            sym.signature = sig_reader.read_string()?;
                        } else {
                            sig_reader.skip_field(sig_wire)?;
                        }
                    }
                }
                (8, WireType::LengthDelimited) => {
                    sym.enclosing_symbol = reader.read_string()?;
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        Ok(sym)
    }

    fn decode_relationship(
        mut reader: ProtobufReader<'_>,
    ) -> Result<ScipRelationship, ProtobufError> {
        let mut rel = ScipRelationship::default();
        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::LengthDelimited) => rel.symbol = reader.read_string()?,
                (2, WireType::Varint) => rel.is_reference = reader.read_varint()? != 0,
                (3, WireType::Varint) => rel.is_implementation = reader.read_varint()? != 0,
                (4, WireType::Varint) => rel.is_type_definition = reader.read_varint()? != 0,
                (5, WireType::Varint) => rel.is_definition = reader.read_varint()? != 0,
                (_, wire) => reader.skip_field(wire)?,
            }
        }
        Ok(rel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protobuf_varint() {
        let data = [0xAC, 0x02];
        let mut reader = ProtobufReader::new(&data);
        assert_eq!(reader.read_varint().unwrap(), 300);
        assert!(reader.is_at_end());
    }

    #[test]
    fn test_scip_range_packed() {
        assert_eq!(
            ScipRange::from_packed(&[10, 5, 20]),
            Some(ScipRange {
                start_line: 10,
                start_col: 5,
                end_line: 10,
                end_col: 20
            })
        );
        assert_eq!(
            ScipRange::from_packed(&[10, 5, 12, 8]),
            Some(ScipRange {
                start_line: 10,
                start_col: 5,
                end_line: 12,
                end_col: 8
            })
        );
    }

    #[test]
    fn test_decode_occurrence_and_symbol_info() {
        // Build Occurrence protobuf:
        // field 1 (wire 2): packed range [1, 2, 1, 10]
        // field 2 (wire 2): string "local 1"
        // field 3 (wire 0): int32 1 (definition)
        // field 7 (wire 2): packed enclosing [1, 0, 5, 1]
        let mut occ_bytes = Vec::new();
        // field 1: tag = (1 << 3) | 2 = 10
        occ_bytes.push(10);
        let range_packed = [1, 2, 1, 10];
        occ_bytes.push(range_packed.len() as u8);
        occ_bytes.extend_from_slice(&range_packed);

        // field 2: tag = (2 << 3) | 2 = 18
        occ_bytes.push(18);
        let sym_str = b"local 1";
        occ_bytes.push(sym_str.len() as u8);
        occ_bytes.extend_from_slice(sym_str);

        // field 3: tag = (3 << 3) | 0 = 24
        occ_bytes.push(24);
        occ_bytes.push(1);

        // field 7: tag = (7 << 3) | 2 = 58
        occ_bytes.push(58);
        let enclosing_packed = [1, 0, 5, 1];
        occ_bytes.push(enclosing_packed.len() as u8);
        occ_bytes.extend_from_slice(&enclosing_packed);

        let occ = ScipIndex::decode_occurrence(ProtobufReader::new(&occ_bytes)).unwrap();
        assert_eq!(occ.symbol, "local 1");
        assert_eq!(occ.symbol_roles, 1);
        assert!(occ.is_definition());
        assert_eq!(
            occ.range,
            Some(ScipRange {
                start_line: 1,
                start_col: 2,
                end_line: 1,
                end_col: 10
            })
        );
        assert_eq!(
            occ.enclosing_range,
            Some(ScipRange {
                start_line: 1,
                start_col: 0,
                end_line: 5,
                end_col: 1
            })
        );

        // Build SymbolInformation protobuf:
        // field 1 (wire 2): symbol "scip-rust/foo"
        // field 5 (wire 0): kind 17 (Function)
        // field 6 (wire 2): display_name "foo"
        // field 7 (wire 2): signature doc with field 5 (wire 2): "fn foo() -> i32"
        // field 8 (wire 2): enclosing_symbol "scip-rust/Bar"
        let mut sym_bytes = Vec::new();
        // field 1
        sym_bytes.push(10);
        let s = b"scip-rust/foo";
        sym_bytes.push(s.len() as u8);
        sym_bytes.extend_from_slice(s);

        // field 5
        sym_bytes.push(5 << 3);
        sym_bytes.push(17);

        // field 6 (display_name)
        sym_bytes.push((6 << 3) | 2);
        let dn = b"foo";
        sym_bytes.push(dn.len() as u8);
        sym_bytes.extend_from_slice(dn);

        // field 7 (signature message)
        let mut sig_msg = Vec::new();
        sig_msg.push((5 << 3) | 2);
        let sig_text = b"fn foo() -> i32";
        sig_msg.push(sig_text.len() as u8);
        sig_msg.extend_from_slice(sig_text);

        sym_bytes.push((7 << 3) | 2);
        sym_bytes.push(sig_msg.len() as u8);
        sym_bytes.extend_from_slice(&sig_msg);

        // field 8 (enclosing_symbol)
        sym_bytes.push((8 << 3) | 2);
        let enc = b"scip-rust/Bar";
        sym_bytes.push(enc.len() as u8);
        sym_bytes.extend_from_slice(enc);

        let info = ScipIndex::decode_symbol_info(ProtobufReader::new(&sym_bytes)).unwrap();
        assert_eq!(info.symbol, "scip-rust/foo");
        assert_eq!(info.kind, 17);
        assert_eq!(info.display_name, "foo");
        assert_eq!(info.signature, "fn foo() -> i32");
        assert_eq!(info.enclosing_symbol, "scip-rust/Bar");
    }
    #[test]
    fn document_and_metadata_encoding_tags_are_preserved() {
        let index = ScipIndex::decode(&[10, 2, 32, 2, 18, 2, 48, 3]).unwrap();
        assert_eq!(index.text_document_encoding, 2);
        assert_eq!(index.documents[0].position_encoding, 3);
    }
    #[test]
    fn packed_ranges_reject_negative_and_reversed_coordinates() {
        for values in [
            &[-1, 0, 1][..],
            &[0, -1, 2],
            &[0, 5, 4],
            &[4, 0, 3, 8],
            &[0, 0, 1, -1],
        ] {
            assert!(
                ScipRange::from_packed(values).is_none(),
                "accepted {values:?}"
            );
        }
        assert!(ScipRange::from_packed(&[0, 1, 1]).is_some());
        let typed_over_packed = ScipIndex::decode_occurrence(ProtobufReader::new(&[
            0x0a, 3, 0, 0, 1, 0x42, 4, 0x10, 5, 0x18, 4,
        ]))
        .unwrap();
        assert!(
            typed_over_packed.range.is_none(),
            "invalid typed coordinates must not fall back to a conflicting packed range"
        );
        assert!(ScipRange::from_packed(&[i32::MAX, 0, 1]).is_some());
        assert!(
            ScipIndex::decode_single_line_range(ProtobufReader::new(&[0x10, 5, 0x18, 4]))
                .unwrap()
                .is_none()
        );
        assert!(
            ScipIndex::decode_multi_line_range(ProtobufReader::new(&[0x08, 4, 0x18, 1]))
                .unwrap()
                .is_none()
        );
    }
}

#[cfg(test)]
mod cancellation_tests;
