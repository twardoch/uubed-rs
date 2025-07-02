// this_file: rust/src/encoders/q64.rs
//! QuadB64: Position-safe encoding with SIMD optimization.

use std::error::Error;
use std::fmt;

/// Position-dependent alphabets
const ALPHABETS: [&[u8; 16]; 4] = [
    b"ABCDEFGHIJKLMNOP",  // pos ≡ 0 (mod 4)
    b"QRSTUVWXYZabcdef",  // pos ≡ 1
    b"ghijklmnopqrstuv",  // pos ≡ 2
    b"wxyz0123456789-_",  // pos ≡ 3
];

/// Reverse lookup table (ASCII char -> (alphabet_idx, nibble_value))
/// We use a const fn to build this at compile time for better performance
const fn build_reverse_lookup() -> [Option<(u8, u8)>; 256] {
    let mut table = [None; 256];
    let mut alphabet_idx = 0;

    // Manual loop unrolling since const fn limitations
    while alphabet_idx < 4 {
        let alphabet = ALPHABETS[alphabet_idx];
        let mut nibble_value = 0;
        while nibble_value < 16 {
            let ch = alphabet[nibble_value];
            table[ch as usize] = Some((alphabet_idx as u8, nibble_value as u8));
            nibble_value += 1;
        }
        alphabet_idx += 1;
    }
    table
}

static REV_LOOKUP: [Option<(u8, u8)>; 256] = build_reverse_lookup();

#[derive(Debug, Clone)]
pub struct Q64Error {
    message: String,
}

impl fmt::Display for Q64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Q64 error: {}", self.message)
    }
}

impl Error for Q64Error {}

/// Encode bytes into Q64 format.
///
/// # Performance
/// - Uses SIMD when available for parallel nibble extraction
/// - Processes 16 bytes at a time on x86_64 with AVX2
/// - Falls back to scalar code on other architectures
pub fn q64_encode(data: &[u8]) -> String {
    let mut result = String::with_capacity(data.len() * 2);

    #[cfg(all(target_arch = "x86_64", feature = "simd"))]
    {
        // SIMD fast path for x86_64
        unsafe { q64_encode_simd(data, &mut result) };
    }
    #[cfg(not(all(target_arch = "x86_64", feature = "simd")))]
    {
        // Scalar fallback
        q64_encode_scalar(data, &mut result);
    }

    result
}

/// Scalar implementation of Q64 encoding
fn q64_encode_scalar(data: &[u8], output: &mut String) {
    for (byte_idx, &byte) in data.iter().enumerate() {
        let hi_nibble = (byte >> 4) & 0xF;
        let lo_nibble = byte & 0xF;
        let base_pos = byte_idx * 2;

        // Use position-dependent alphabets
        output.push(ALPHABETS[base_pos & 3][hi_nibble as usize] as char);
        output.push(ALPHABETS[(base_pos + 1) & 3][lo_nibble as usize] as char);
    }
}

/// SIMD implementation for x86_64 with AVX2
#[cfg(all(target_arch = "x86_64", feature = "simd"))]
unsafe fn q64_encode_simd(data: &[u8], output: &mut String) {
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let chunks = data.chunks_exact(16);
    let remainder = chunks.remainder();

    // Process 16 bytes at a time
    for (chunk_idx, chunk) in chunks.enumerate() {
        // Load 16 bytes
        let input = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);

        // Split into high and low nibbles
        let lo_mask = _mm_set1_epi8(0x0F);

        let hi_nibbles = _mm_and_si128(_mm_srli_epi16(input, 4), lo_mask);
        let lo_nibbles = _mm_and_si128(input, lo_mask);

        // Process nibbles and convert to characters
        let base_pos = chunk_idx * 32;

        // Extract and encode each nibble
        for i in 0..16 {
            let hi = _mm_extract_epi8(hi_nibbles, i) as usize;
            let lo = _mm_extract_epi8(lo_nibbles, i) as usize;

            let pos = base_pos + i * 2;
            output.push(ALPHABETS[pos & 3][hi] as char);
            output.push(ALPHABETS[(pos + 1) & 3][lo] as char);
        }
    }

    // Handle remainder with scalar code
    let byte_offset = data.len() - remainder.len();
    for (idx, &byte) in remainder.iter().enumerate() {
        let byte_idx = byte_offset + idx;
        let hi_nibble = (byte >> 4) & 0xF;
        let lo_nibble = byte & 0xF;
        let base_pos = byte_idx * 2;

        output.push(ALPHABETS[base_pos & 3][hi_nibble as usize] as char);
        output.push(ALPHABETS[(base_pos + 1) & 3][lo_nibble as usize] as char);
    }
}

/// Decode Q64 string back to bytes
pub fn q64_decode(encoded: &str) -> Result<Vec<u8>, Q64Error> {
    if encoded.len() & 1 != 0 {
        return Err(Q64Error {
            message: "Q64 string length must be even".to_string(),
        });
    }

    let mut result = Vec::with_capacity(encoded.len() / 2);
    let chars: Vec<char> = encoded.chars().collect();

    for (pos, chunk) in chars.chunks_exact(2).enumerate() {
        let ch1 = chunk[0];
        let ch2 = chunk[1];

        // Validate and decode first nibble
        let (_, nibble1) = validate_char(ch1, pos * 2)?;

        // Validate and decode second nibble
        let (_, nibble2) = validate_char(ch2, pos * 2 + 1)?;

        // Combine nibbles into byte
        result.push((nibble1 << 4) | nibble2);
    }

    Ok(result)
}

/// Validate character and return (alphabet_idx, nibble_value)
fn validate_char(ch: char, pos: usize) -> Result<(u8, u8), Q64Error> {
    if ch as u32 > 255 {
        return Err(Q64Error {
            message: format!("Non-ASCII character '{}' at position {}", ch, pos),
        });
    }

    match REV_LOOKUP[ch as usize] {
        Some((alphabet_idx, nibble_value)) => {
            let expected_alphabet = (pos & 3) as u8;
            if alphabet_idx != expected_alphabet {
                Err(Q64Error {
                    message: format!(
                        "Character '{}' from alphabet {} at position {} (expected alphabet {})",
                        ch, alphabet_idx, pos, expected_alphabet
                    ),
                })
            } else {
                Ok((alphabet_idx, nibble_value))
            }
        }
        None => Err(Q64Error {
            message: format!("Invalid Q64 character '{}' at position {}", ch, pos),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let data = vec![0, 127, 255, 42, 100];
        let encoded = q64_encode(&data);
        let decoded = q64_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_position_safety() {
        let data = vec![0, 0, 0, 0];
        let encoded = q64_encode(&data);

        // Verify each character is from correct alphabet
        for (i, ch) in encoded.chars().enumerate() {
            let alphabet_idx = i & 3;
            assert!(ALPHABETS[alphabet_idx].contains(&(ch as u8)));
        }
    }

    #[test]
    fn test_empty() {
        let data = vec![];
        let encoded = q64_encode(&data);
        assert_eq!(encoded, "");
        let decoded = q64_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_error_odd_length() {
        assert!(q64_decode("ABC").is_err());
    }

    #[test]
    fn test_error_invalid_char() {
        assert!(q64_decode("!@").is_err());
    }

    #[test]
    fn test_error_wrong_position() {
        assert!(q64_decode("QA").is_err());
    }
}