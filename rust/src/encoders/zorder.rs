// this_file: rust/src/encoders/zorder.rs
//! Z-order (Morton code) encoder for spatial locality.

/// Interleave bits for Z-order curve
///
/// This creates a space-filling curve that preserves spatial locality.
/// Points that are close in high-dimensional space will have similar
/// Z-order codes and thus similar prefixes.
pub fn z_order_q64(embedding: &[u8]) -> String {
    // Take top 2 bits from each dimension
    let quantized: Vec<u8> = embedding
        .iter()
        .map(|&b| (b >> 6) & 0b11)
        .collect();

    // We'll interleave bits from up to 16 dimensions into a 32-bit value
    let dims_to_use = quantized.len().min(16);
    let mut result: u32 = 0;

    // Bit interleaving using bit manipulation tricks
    for dim in 0..dims_to_use {
        let val = quantized[dim] as u32;

        // Spread the 2 bits across the result
        // Bit 0 goes to position dim*2
        // Bit 1 goes to position dim*2 + 1
        result |= (val & 0b01) << (dim * 2);
        result |= ((val & 0b10) >> 1) << (dim * 2 + 1);
    }

    // Convert to bytes
    let bytes = result.to_be_bytes();
    super::q64::q64_encode(&bytes)
}

/// Advanced Z-order with more bits per dimension
///
/// This version uses 4 bits per dimension for finer granularity
pub fn z_order_q64_extended(embedding: &[u8]) -> String {
    // Take top 4 bits from each dimension
    let quantized: Vec<u8> = embedding
        .iter()
        .map(|&b| (b >> 4) & 0b1111)
        .collect();

    // We can fit 8 dimensions × 4 bits = 32 bits
    let dims_to_use = quantized.len().min(8);
    let mut result: u32 = 0;

    // Interleave 4 bits from each dimension
    for dim in 0..dims_to_use {
        let val = quantized[dim] as u32;

        // Use bit manipulation to spread bits
        // This is a simplified version - production code would use
        // lookup tables or PDEP instruction for efficiency
        for bit in 0..4 {
            let bit_val = (val >> bit) & 1;
            result |= bit_val << (bit * 8 + dim);
        }
    }

    // Convert to bytes
    let bytes = result.to_be_bytes();
    super::q64::q64_encode(&bytes)
}

/// Fast Z-order using lookup tables
/// For production use, this would be the preferred method
#[cfg(feature = "simd")]
pub fn z_order_q64_fast(embedding: &[u8]) -> String {
    // Lookup tables for fast bit interleaving
    // Pre-computed Morton codes for 2-bit values
    const MORTON_TABLE_X: [u32; 4] = [0b00, 0b01, 0b100, 0b101];
    const MORTON_TABLE_Y: [u32; 4] = [0b00, 0b10, 0b1000, 0b1010];

    let quantized: Vec<u8> = embedding
        .iter()
        .map(|&b| (b >> 6) & 0b11)
        .collect();

    let mut result: u32 = 0;

    // Process pairs of dimensions
    for i in (0..quantized.len().min(16)).step_by(2) {
        let x = quantized[i] as usize;
        let y = quantized.get(i + 1).copied().unwrap_or(0) as usize;

        let morton = MORTON_TABLE_X[x] | MORTON_TABLE_Y[y];
        result |= morton << (i * 2);
    }

    let bytes = result.to_be_bytes();
    super::q64::q64_encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_order_basic() {
        // Test that similar inputs produce similar codes
        let vec1 = vec![255, 255, 0, 0];  // Top-left in 2D
        let vec2 = vec![255, 254, 0, 0];  // Very close to vec1
        let vec3 = vec![0, 0, 255, 255];  // Bottom-right in 2D

        let z1 = z_order_q64(&vec1);
        let z2 = z_order_q64(&vec2);
        let z3 = z_order_q64(&vec3);

        // z1 and z2 should share a longer prefix than z1 and z3
        let prefix_len_12 = z1.chars()
            .zip(z2.chars())
            .take_while(|(a, b)| a == b)
            .count();

        let prefix_len_13 = z1.chars()
            .zip(z3.chars())
            .take_while(|(a, b)| a == b)
            .count();

        assert!(prefix_len_12 > prefix_len_13);
    }

    #[test]
    fn test_z_order_length() {
        let data = vec![0; 32];
        let encoded = z_order_q64(&data);
        assert_eq!(encoded.len(), 8); // 4 bytes = 8 q64 chars
    }

    #[test]
    fn test_z_order_empty() {
        let data = vec![];
        let encoded = z_order_q64(&data);
        assert_eq!(encoded.len(), 8); // Still 4 bytes of zeros
    }

    #[test]
    fn test_z_order_extended() {
        let data = vec![0xFF, 0xF0, 0x0F, 0x00];
        let basic = z_order_q64(&data);
        let extended = z_order_q64_extended(&data);

        // Both should be 8 chars
        assert_eq!(basic.len(), 8);
        assert_eq!(extended.len(), 8);

        // Extended should capture more detail
        assert_ne!(basic, extended);
    }

    #[cfg(feature = "simd")]
    #[test]
    fn test_z_order_fast() {
        let data = vec![255, 192, 128, 64];
        let basic = z_order_q64(&data);
        let fast = z_order_q64_fast(&data);

        // Should produce same result
        assert_eq!(basic, fast);
    }
}