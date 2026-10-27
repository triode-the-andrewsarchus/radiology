use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProtobufError {
    #[error("Truncated protobuf data")]
    Truncated,
    #[error("Malformed varint")]
    MalformedVarint,
    #[error("Unsupported wire type: {0}")]
    UnsupportedWireType(u8),
    #[error("Invalid UTF-8 string")]
    InvalidUtf8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    Fixed32 = 5,
}

impl WireType {
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

pub struct ProtobufReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ProtobufReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    pub fn is_at_end(&self) -> bool {
        self.offset >= self.data.len()
    }

    pub fn read_tag(&mut self) -> Result<Option<(u32, WireType)>, ProtobufError> {
        if self.is_at_end() {
            return Ok(None);
        }
        let key = self.read_varint()?;
        let raw_wire = (key & 0x7) as u8;
        let wire = WireType::from_u8(raw_wire)?;
        let field = (key >> 3) as u32;
        Ok(Some((field, wire)))
    }

    pub fn read_varint(&mut self) -> Result<u64, ProtobufError> {
        let mut result = 0u64;
        let mut shift = 0u32;
        while self.offset < self.data.len() {
            let byte = self.data[self.offset];
            self.offset += 1;
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

    pub fn read_int32(&mut self) -> Result<i32, ProtobufError> {
        Ok(self.read_varint()? as i32)
    }

    pub fn read_length_delimited(&mut self) -> Result<&'a [u8], ProtobufError> {
        let len = self.read_varint()? as usize;
        if self.offset + len > self.data.len() {
            return Err(ProtobufError::Truncated);
        }
        let slice = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Ok(slice)
    }

    pub fn read_string(&mut self) -> Result<String, ProtobufError> {
        let bytes = self.read_length_delimited()?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ProtobufError::InvalidUtf8)
    }

    pub fn read_packed_int32(&mut self) -> Result<Vec<i32>, ProtobufError> {
        let bytes = self.read_length_delimited()?;
        let mut reader = ProtobufReader::new(bytes);
        let mut out = Vec::new();
        while !reader.is_at_end() {
            out.push(reader.read_int32()?);
        }
        Ok(out)
    }

    pub fn skip_field(&mut self, wire: WireType) -> Result<(), ProtobufError> {
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
                let len = self.read_varint()? as usize;
                if self.offset + len > self.data.len() {
                    return Err(ProtobufError::Truncated);
                }
                self.offset += len;
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

// SCIP schema models
#[derive(Debug, Clone, Default)]
pub struct ScipIndex {
    pub documents: Vec<ScipDocument>,
    pub external_symbols: Vec<ScipSymbolInformation>,
}

#[derive(Debug, Clone, Default)]
pub struct ScipDocument {
    pub relative_path: String,
    pub language: String,
    pub occurrences: Vec<ScipOccurrence>,
    pub symbols: Vec<ScipSymbolInformation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScipRange {
    pub start_line: i32,
    pub start_col: i32,
    pub end_line: i32,
    pub end_col: i32,
}

impl ScipRange {
    pub fn from_packed(vals: &[i32]) -> Option<Self> {
        match vals.len() {
            3 => Some(Self {
                start_line: vals[0],
                start_col: vals[1],
                end_line: vals[0],
                end_col: vals[2],
            }),
            4 => Some(Self {
                start_line: vals[0],
                start_col: vals[1],
                end_line: vals[2],
                end_col: vals[3],
            }),
            _ => None,
        }
    }
}

pub const SCIP_ROLE_DEFINITION: i32 = 1 << 0;
pub const SCIP_ROLE_IMPORT: i32 = 1 << 1;
pub const SCIP_ROLE_WRITE: i32 = 1 << 2;
pub const SCIP_ROLE_READ: i32 = 1 << 3;

#[derive(Debug, Clone, Default)]
pub struct ScipOccurrence {
    pub range: Option<ScipRange>,
    pub symbol: String,
    pub symbol_roles: i32,
    pub enclosing_range: Option<ScipRange>,
}

impl ScipOccurrence {
    pub fn is_definition(&self) -> bool {
        (self.symbol_roles & SCIP_ROLE_DEFINITION) != 0
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScipSymbolInformation {
    pub symbol: String,
    pub documentation: Vec<String>,
    pub display_name: String,
}

impl ScipIndex {
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtobufError> {
        let mut reader = ProtobufReader::new(bytes);
        let mut index = ScipIndex::default();

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (2, WireType::LengthDelimited) => {
                    let doc_bytes = reader.read_length_delimited()?;
                    index.documents.push(Self::decode_document(doc_bytes)?);
                }
                (3, WireType::LengthDelimited) => {
                    let sym_bytes = reader.read_length_delimited()?;
                    index
                        .external_symbols
                        .push(Self::decode_symbol_info(sym_bytes)?);
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        Ok(index)
    }

    fn decode_document(bytes: &[u8]) -> Result<ScipDocument, ProtobufError> {
        let mut reader = ProtobufReader::new(bytes);
        let mut doc = ScipDocument::default();

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::LengthDelimited) => {
                    doc.relative_path = reader.read_string()?;
                }
                (2, WireType::LengthDelimited) => {
                    let occ_bytes = reader.read_length_delimited()?;
                    doc.occurrences.push(Self::decode_occurrence(occ_bytes)?);
                }
                (3, WireType::LengthDelimited) => {
                    let sym_bytes = reader.read_length_delimited()?;
                    doc.symbols.push(Self::decode_symbol_info(sym_bytes)?);
                }
                (4, WireType::LengthDelimited) => {
                    doc.language = reader.read_string()?;
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        Ok(doc)
    }

    fn decode_occurrence(bytes: &[u8]) -> Result<ScipOccurrence, ProtobufError> {
        let mut reader = ProtobufReader::new(bytes);
        let mut occ = ScipOccurrence::default();

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::LengthDelimited) => {
                    let vals = reader.read_packed_int32()?;
                    occ.range = ScipRange::from_packed(&vals);
                }
                (2, WireType::LengthDelimited) => {
                    occ.symbol = reader.read_string()?;
                }
                (3, WireType::Varint) => {
                    occ.symbol_roles = reader.read_int32()?;
                }
                (5, WireType::LengthDelimited) => {
                    let vals = reader.read_packed_int32()?;
                    occ.enclosing_range = ScipRange::from_packed(&vals);
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        Ok(occ)
    }

    fn decode_symbol_info(bytes: &[u8]) -> Result<ScipSymbolInformation, ProtobufError> {
        let mut reader = ProtobufReader::new(bytes);
        let mut sym = ScipSymbolInformation::default();

        while let Some((field, wire)) = reader.read_tag()? {
            match (field, wire) {
                (1, WireType::LengthDelimited) => {
                    sym.symbol = reader.read_string()?;
                }
                (3, WireType::LengthDelimited) => {
                    sym.documentation.push(reader.read_string()?);
                }
                (7, WireType::LengthDelimited) => {
                    sym.display_name = reader.read_string()?;
                }
                (_, wire) => reader.skip_field(wire)?,
            }
        }

        Ok(sym)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protobuf_varint() {
        // Varint 300 = 0xAC 0x02
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
}
