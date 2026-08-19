//! Schema-driven binary serialization for LITHOS

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use bytemuck::{Pod, Zeroable, cast_slice, cast_slice_mut};
use crate::lithos_engine_memory::GLOBAL_STATS;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("Schema not found: {0}")]
    SchemaNotFound(String),
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
    #[error("Buffer too small: need {need}, have {have}")]
    BufferTooSmall { need: usize, have: usize },
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    #[error("Unsupported type: {0}")]
    UnsupportedType(String),
}

/// Magic bytes for file format identification
pub const MAGIC: &[u8; 4] = b"LTHS";
pub const CURRENT_VERSION: u32 = 1;

/// Type identifier for schema registry
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TypeId(pub u64);

impl TypeId {
    pub fn new(name: &str) -> Self {
        // FNV-1a hash
        let mut hash: u64 = 0xcbf29ce484222325;
        for &b in name.as_bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self(hash)
    }
}

/// Schema definition for a type
#[derive(Clone, Debug)]
pub struct Schema {
    pub type_id: TypeId,
    pub name: String,
    pub version: u32,
    pub size: usize,
    pub align: usize,
    pub fields: Vec<Field>,
    pub migration: Option<MigrationFn>,
}

type MigrationFn = Arc<dyn Fn(&mut [u8], u32, u32) -> Result<(), SerializationError> + Send + Sync>;

#[derive(Clone, Debug)]
pub struct Field {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub type_id: TypeId,
    pub is_array: bool,
    pub array_len: usize,
}

/// Schema registry
pub struct SchemaRegistry {
    schemas: RwLock<HashMap<TypeId, Schema>>,
    by_name: RwLock<HashMap<String, TypeId>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: RwLock::new(HashMap::new()),
            by_name: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, schema: Schema) -> Result<(), SerializationError> {
        let type_id = schema.type_id;
        let name = schema.name.clone();

        let mut schemas = self.schemas.write().unwrap();
        let mut by_name = self.by_name.write().unwrap();

        if schemas.contains_key(&type_id) {
            return Err(SerializationError::InvalidData(format!("Schema already registered: {}", name)));
        }

        schemas.insert(type_id, schema);
        by_name.insert(name, type_id);
        Ok(())
    }

    pub fn get(&self, type_id: TypeId) -> Option<Schema> {
        self.schemas.read().unwrap().get(&type_id).cloned()
    }

    pub fn get_by_name(&self, name: &str) -> Option<Schema> {
        let by_name = self.by_name.read().unwrap();
        let type_id = by_name.get(name)?;
        self.schemas.read().unwrap().get(type_id).cloned()
    }

    pub fn list_types(&self) -> Vec<String> {
        self.by_name.read().unwrap().keys().cloned().collect()
    }
}

/// Global schema registry
static GLOBAL_REGISTRY: std::sync::OnceLock<SchemaRegistry> = std::sync::OnceLock::new();

pub fn global_registry() -> &'static SchemaRegistry {
    GLOBAL_REGISTRY.get_or_init(SchemaRegistry::new)
}

/// Register built-in schemas
pub fn register_builtin_schemas() {
    let registry = global_registry();

    // Register primitive types
    macro_rules! register_primitive {
        ($ty:ty, $name:expr) => {{
            let schema = Schema {
                type_id: TypeId::new($name),
                name: $name.to_string(),
                version: 1,
                size: std::mem::size_of::<$ty>(),
                align: std::mem::align_of::<$ty>(),
                fields: vec![],
                migration: None,
            };
            registry.register(schema).unwrap();
        }};
    }

    register_primitive!(u8, "u8");
    register_primitive!(u16, "u16");
    register_primitive!(u32, "u32");
    register_primitive!(u64, "u64");
    register_primitive!(i8, "i8");
    register_primitive!(i16, "i16");
    register_primitive!(i32, "i32");
    register_primitive!(i64, "i64");
    register_primitive!(f32, "f32");
    register_primitive!(f64, "f64");
    register_primitive!(bool, "bool");
}

/// Serialize trait for types that can be serialized
pub trait Serialize {
    fn type_id() -> TypeId;
    fn schema_version() -> u32;
    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), SerializationError>;
    fn deserialize(buf: &[u8]) -> Result<Self, SerializationError> where Self: Sized;
}

/// Serialize Pod types directly
impl<T: Pod + Zeroable + 'static> Serialize for T {
    fn type_id() -> TypeId {
        TypeId::new(std::any::type_name::<T>())
    }

    fn schema_version() -> u32 {
        1
    }

    fn serialize(&self, buf: &mut Vec<u8>) -> Result<(), SerializationError> {
        let bytes = cast_slice(std::slice::from_ref(self));
        buf.extend_from_slice(bytes);
        Ok(())
    }

    fn deserialize(buf: &[u8]) -> Result<Self, SerializationError> {
        if buf.len() < std::mem::size_of::<T>() {
            return Err(SerializationError::BufferTooSmall {
                need: std::mem::size_of::<T>(),
                have: buf.len(),
            });
        }
        let val = *cast_slice::<u8, T>(buf).first().unwrap();
        Ok(val)
    }
}

/// Binary writer
pub struct Writer {
    buf: Vec<u8>,
    pos: usize,
}

impl Writer {
    pub fn new() -> Self {
        Self { buf: Vec::new(), pos: 0 }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self { buf: Vec::with_capacity(cap), pos: 0 }
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buf.extend_from_slice(data);
        self.pos += data.len();
    }

    pub fn write<T: Pod>(&mut self, value: &T) {
        let bytes = cast_slice(std::slice::from_ref(value));
        self.write_bytes(bytes);
    }

    pub fn write_varint(&mut self, mut value: u64) {
        while value >= 0x80 {
            self.buf.push((value & 0x7F) as u8 | 0x80);
            value >>= 7;
            self.pos += 1;
        }
        self.buf.push(value as u8);
        self.pos += 1;
    }

    pub fn write_zigzag(&mut self, value: i64) {
        self.write_varint(((value << 1) ^ (value >> 63)) as u64);
    }

    pub fn write_string(&mut self, s: &str) {
        self.write_varint(s.len() as u64);
        self.write_bytes(s.as_bytes());
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn len(&self) -> usize {
        self.pos
    }
}

/// Binary reader
pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], SerializationError> {
        if self.pos + len > self.buf.len() {
            return Err(SerializationError::BufferTooSmall {
                need: self.pos + len,
                have: self.buf.len(),
            });
        }
        let data = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(data)
    }

    pub fn read<T: Pod + Zeroable>(&mut self) -> Result<T, SerializationError> {
        let size = std::mem::size_of::<T>();
        let bytes = self.read_bytes(size)?;
        let val = *cast_slice::<u8, T>(bytes).first().unwrap();
        Ok(val)
    }

    pub fn read_varint(&mut self) -> Result<u64, SerializationError> {
        let mut result = 0u64;
        let mut shift = 0;
        loop {
            if self.pos >= self.buf.len() {
                return Err(SerializationError::BufferTooSmall { need: self.pos + 1, have: self.buf.len() });
            }
            let byte = self.buf[self.pos];
            self.pos += 1;
            result |= ((byte & 0x7F) as u64) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
            if shift >= 64 {
                return Err(SerializationError::InvalidData("Varint too long".to_string()));
            }
        }
        Ok(result)
    }

    pub fn read_zigzag(&mut self) -> Result<i64, SerializationError> {
        let n = self.read_varint()?;
        Ok(((n >> 1) as i64) ^ (-((n & 1) as i64)))
    }

    pub fn read_string(&mut self) -> Result<String, SerializationError> {
        let len = self.read_varint()? as usize;
        let bytes = self.read_bytes(len)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.pos
    }
}

/// File header
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub schema_type: u64,
    pub schema_version: u32,
    pub flags: u32,
    pub payload_size: u64,
    pub checksum: u64,
}

impl FileHeader {
    pub fn new(schema_type: TypeId, schema_version: u32, payload_size: u64) -> Self {
        Self {
            magic: *MAGIC,
            version: CURRENT_VERSION,
            schema_type: schema_type.0,
            schema_version,
            flags: 0,
            payload_size,
            checksum: 0, // TODO: compute
        }
    }

    pub fn validate(&self) -> Result<(), SerializationError> {
        if self.magic != *MAGIC {
            return Err(SerializationError::InvalidData("Invalid magic".to_string()));
        }
        if self.version != CURRENT_VERSION {
            return Err(SerializationError::VersionMismatch {
                expected: CURRENT_VERSION,
                actual: self.version,
            });
        }
        Ok(())
    }
}

/// Serialize to bytes with header
pub fn serialize_with_header<T: Serialize>(value: &T) -> Result<Vec<u8>, SerializationError> {
    let mut payload = Vec::new();
    value.serialize(&mut payload)?;

    let header = FileHeader::new(
        T::type_id(),
        T::schema_version(),
        payload.len() as u64,
    );

    let mut writer = Writer::new();
    writer.write(&header);
    writer.write_bytes(&payload);

    Ok(writer.into_bytes())
}

/// Deserialize from bytes with header
pub fn deserialize_with_header<T: Serialize>(buf: &[u8]) -> Result<T, SerializationError> {
    let mut reader = Reader::new(buf);
    let header: FileHeader = reader.read()?;
    header.validate()?;

    if header.schema_type != T::type_id().0 {
        return Err(SerializationError::InvalidData("Schema type mismatch".to_string()));
    }

    if header.schema_version != T::schema_version() {
        // TODO: Run migration
        return Err(SerializationError::VersionMismatch {
            expected: T::schema_version(),
            actual: header.schema_version,
        });
    }

    T::deserialize(reader.buf)
}

/// Migration trait
pub trait Migrate {
    fn migrate(data: &mut [u8], from_version: u32, to_version: u32) -> Result<(), SerializationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_serialization() {
        let val: u32 = 42;
        let mut buf = Vec::new();
        val.serialize(&mut buf).unwrap();
        let result = u32::deserialize(&buf).unwrap();
        assert_eq!(val, result);
    }

    #[test]
    fn test_varint() {
        let mut writer = Writer::new();
        writer.write_varint(0);
        writer.write_varint(127);
        writer.write_varint(128);
        writer.write_varint(16383);
        writer.write_varint(16384);

        let mut reader = Reader::new(&writer.buf);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 127);
        assert_eq!(reader.read_varint().unwrap(), 128);
        assert_eq!(reader.read_varint().unwrap(), 16383);
        assert_eq!(reader.read_varint().unwrap(), 16384);
    }

    #[test]
    fn test_zigzag() {
        let mut writer = Writer::new();
        writer.write_zigzag(0);
        writer.write_zigzag(-1);
        writer.write_zigzag(1);
        writer.write_zigzag(-2);
        writer.write_zigzag(2);

        let mut reader = Reader::new(&writer.buf);
        assert_eq!(reader.read_zigzag().unwrap(), 0);
        assert_eq!(reader.read_zigzag().unwrap(), -1);
        assert_eq!(reader.read_zigzag().unwrap(), 1);
        assert_eq!(reader.read_zigzag().unwrap(), -2);
        assert_eq!(reader.read_zigzag().unwrap(), 2);
    }

    #[test]
    fn test_string() {
        let mut writer = Writer::new();
        writer.write_string("hello");
        writer.write_string("");

        let mut reader = Reader::new(&writer.buf);
        assert_eq!(reader.read_string().unwrap(), "hello");
        assert_eq!(reader.read_string().unwrap(), "");
    }

    #[test]
    fn test_header_roundtrip() {
        let val = 12345u64;
        let bytes = serialize_with_header(&val).unwrap();
        let result = deserialize_with_header::<u64>(&bytes).unwrap();
        assert_eq!(val, result);
    }
}