// this_file: rust/src/encoders/mq64.rs
//! Matryoshka QuadB64 (Mq64): Hierarchical position-safe encoding prototype
use crate::encoders::q64_encode;
use crate::encoders::q64_decode;
use crate::encoders::q64::Q64Error;

/// Encode data into Mq64 string using default hierarchical levels (powers of two up to full length)
pub fn mq64_encode(data: &[u8]) -> String {
    // Determine default levels: 64, 128, 256, ... up to data.len()
    let mut levels = Vec::new();
    let mut level = 64;
    while level < data.len() {
        levels.push(level);
        level *= 2;
    }
    levels.push(data.len());
    mq64_encode_with_levels(data, &levels)
}

/// Encode data into Mq64 string with explicit hierarchical levels
pub fn mq64_encode_with_levels(data: &[u8], levels: &[usize]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for &lvl in levels {
        if lvl <= data.len() {
            let part = q64_encode(&data[..lvl]);
            parts.push(part);
        }
    }
    parts.join(":")
}

/// Decode Mq64 string, returning full-data bytes (last level)
pub fn mq64_decode(encoded: &str) -> Result<Vec<u8>, Q64Error> {
    let segments: Vec<&str> = encoded.split(':').collect();
    if let Some(last) = segments.last() {
        q64_decode(last)
    } else {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mq64_roundtrip_default() {
        let data = vec![0u8; 128];
        let encoded = mq64_encode(&data);
        // Should contain at least one separator for data.len()>64
        assert!(encoded.contains(':'));
        let decoded = mq64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_mq64_with_levels_explicit() {
        let data: Vec<u8> = (0..200u8).collect();
        let levels = vec![64, 128, 200];
        let encoded = mq64_encode_with_levels(&data, &levels);
        let parts: Vec<&str> = encoded.split(':').collect();
        assert_eq!(parts.len(), 3);
        // Decode full layer
        let decoded = mq64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}