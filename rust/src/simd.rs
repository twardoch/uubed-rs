// this_file: rust/src/simd.rs
/// SIMD optimizations for various architectures

// Error types available for future use
#[allow(unused_imports)]
use crate::error::{UubedError, UubedResult};

/// SIMD capabilities detection and dispatch
pub mod dispatch {
    /// Available SIMD instruction sets
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SimdLevel {
        Scalar,
        Sse2,
        Sse41,
        Avx2,
        Avx512,
        Neon,
    }
    
    /// Runtime detection of available SIMD capabilities
    pub fn detect_simd_level() -> SimdLevel {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw") {
                SimdLevel::Avx512
            } else if is_x86_feature_detected!("avx2") {
                SimdLevel::Avx2
            } else if is_x86_feature_detected!("sse4.1") {
                SimdLevel::Sse41
            } else if is_x86_feature_detected!("sse2") {
                SimdLevel::Sse2
            } else {
                SimdLevel::Scalar
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                SimdLevel::Neon
            } else {
                SimdLevel::Scalar
            }
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            SimdLevel::Scalar
        }
    }
    
    /// Get human-readable name for SIMD level
    pub fn simd_level_name(level: SimdLevel) -> &'static str {
        match level {
            SimdLevel::Scalar => "Scalar",
            SimdLevel::Sse2 => "SSE2",
            SimdLevel::Sse41 => "SSE4.1",
            SimdLevel::Avx2 => "AVX2",
            SimdLevel::Avx512 => "AVX-512",
            SimdLevel::Neon => "NEON",
        }
    }
}

/// SIMD-optimized Q64 encoding
pub mod q64_simd {
    use super::dispatch::SimdLevel;
    #[allow(unused_imports)]
    use crate::error::{UubedError, UubedResult, Q64ErrorKind};
    
    /// Dispatch Q64 encoding to best available SIMD implementation
    pub fn q64_encode_simd_dispatch(data: &[u8]) -> String {
        let simd_level = super::dispatch::detect_simd_level();
        
        match simd_level {
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx512 => unsafe { q64_encode_avx512(data) },
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 => unsafe { q64_encode_avx2(data) },
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Sse41 | SimdLevel::Sse2 => unsafe { q64_encode_sse2(data) },
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => unsafe { q64_encode_neon(data) },
            _ => q64_encode_scalar(data),
        }
    }
    
    /// Scalar implementation (fallback)
    fn q64_encode_scalar(data: &[u8]) -> String {
        // This would call the existing scalar implementation
        crate::encoders::q64_encode(data)
    }
    
    /// AVX-512 optimized Q64 encoding (disabled due to unstable intrinsics)
    #[cfg(target_arch = "x86_64")]
    #[allow(dead_code)]
    unsafe fn q64_encode_avx512(data: &[u8]) -> String {
        // AVX-512 requires nightly Rust, fall back to AVX2 for now
        q64_encode_avx2(data)
    }
    
    /// AVX2 optimized Q64 encoding
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn q64_encode_avx2(data: &[u8]) -> String {
        #[cfg(target_arch = "x86_64")]
        #[allow(unused_imports)]
        use std::arch::x86_64::*;
        
        let mut output = String::with_capacity(data.len() * 2);
        let alphabets = get_q64_alphabets();
        
        // For small inputs, use scalar implementation
        if data.len() < 32 {
            for (idx, &byte) in data.iter().enumerate() {
                encode_byte_scalar(byte, idx, &mut output, &alphabets);
            }
            return output;
        }
        
        // Process 16 bytes at a time for better performance
        // (32 bytes requires too much register pressure)
        let chunks = data.chunks_exact(16);
        let remainder = chunks.remainder();
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            // Load 16 bytes into lower half of 256-bit register
            let input_128 = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let input = _mm256_castsi128_si256(input_128);
            
            // Split into high and low nibbles
            let lo_mask = _mm256_set1_epi8(0x0F);
            let hi_nibbles = _mm256_and_si256(_mm256_srli_epi16(input, 4), lo_mask);
            let lo_nibbles = _mm256_and_si256(input, lo_mask);
            
            // Process nibbles efficiently
            let base_pos = chunk_idx * 32; // 16 bytes * 2 chars per byte
            
            // Convert to array for extraction (SIMD intrinsics require constant indices)
            let hi_array: [u8; 32] = std::mem::transmute(hi_nibbles);
            let lo_array: [u8; 32] = std::mem::transmute(lo_nibbles);
            
            // Process nibbles in pairs for alphabet efficiency
            for i in 0..16 {
                let hi = hi_array[i];
                let lo = lo_array[i];
                
                let alphabet_idx_hi = (base_pos + i * 2) & 3;
                let alphabet_idx_lo = (base_pos + i * 2 + 1) & 3;
                
                output.push(alphabets[alphabet_idx_hi][hi as usize] as char);
                output.push(alphabets[alphabet_idx_lo][lo as usize] as char);
            }
        }
        
        // Handle remainder with scalar implementation
        for (idx, &byte) in remainder.iter().enumerate() {
            let byte_idx = data.len() - remainder.len() + idx;
            encode_byte_scalar(byte, byte_idx, &mut output, &alphabets);
        }
        
        output
    }
    
    /// SSE2 optimized Q64 encoding
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn q64_encode_sse2(data: &[u8]) -> String {
        #[cfg(target_arch = "x86_64")]
        #[allow(unused_imports)]
        use std::arch::x86_64::*;
        
        let mut output = String::with_capacity(data.len() * 2);
        let alphabets = get_q64_alphabets();
        
        // Process 16 bytes at a time with SSE2
        let chunks = data.chunks_exact(16);
        let remainder = chunks.remainder();
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            // Load 16 bytes
            let input = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            
            // Split into high and low nibbles
            let lo_mask = _mm_set1_epi8(0x0F);
            let hi_nibbles = _mm_and_si128(_mm_srli_epi16(input, 4), lo_mask);
            let lo_nibbles = _mm_and_si128(input, lo_mask);
            
            // Extract and encode each nibble (use scalar extraction for simplicity)
            let base_pos = chunk_idx * 32; // 16 bytes * 2 chars per byte
            
            // Convert SIMD registers to arrays for scalar processing
            let hi_array: [u8; 16] = std::mem::transmute(hi_nibbles);
            let lo_array: [u8; 16] = std::mem::transmute(lo_nibbles);
            
            for i in 0..16 {
                let hi = hi_array[i] as usize;
                let lo = lo_array[i] as usize;
                
                let pos = base_pos + i * 2;
                output.push(alphabets[pos & 3][hi] as char);
                output.push(alphabets[(pos + 1) & 3][lo] as char);
            }
        }
        
        // Handle remainder
        for (idx, &byte) in remainder.iter().enumerate() {
            let byte_idx = data.len() - remainder.len() + idx;
            encode_byte_scalar(byte, byte_idx, &mut output, &alphabets);
        }
        
        output
    }
    
    /// NEON optimized Q64 encoding (ARM64)
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn q64_encode_neon(data: &[u8]) -> String {
        #[cfg(target_arch = "aarch64")]
        use std::arch::aarch64::*;
        
        let mut output = String::with_capacity(data.len() * 2);
        let alphabets = get_q64_alphabets();
        
        // Process 16 bytes at a time with NEON
        let chunks = data.chunks_exact(16);
        let remainder = chunks.remainder();
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            // Load 16 bytes
            let input = vld1q_u8(chunk.as_ptr());
            
            // Split into high and low nibbles
            let lo_mask = vdupq_n_u8(0x0F);
            let hi_nibbles = vandq_u8(vshrq_n_u8(input, 4), lo_mask);
            let lo_nibbles = vandq_u8(input, lo_mask);
            
            // Extract and encode each nibble
            let base_pos = chunk_idx * 32;
            
            let hi_array: [u8; 16] = std::mem::transmute(hi_nibbles);
            let lo_array: [u8; 16] = std::mem::transmute(lo_nibbles);
            
            for i in 0..16 {
                let hi = hi_array[i] as usize;
                let lo = lo_array[i] as usize;
                
                let pos = base_pos + i * 2;
                output.push(alphabets[pos & 3][hi] as char);
                output.push(alphabets[(pos + 1) & 3][lo] as char);
            }
        }
        
        // Handle remainder
        for (idx, &byte) in remainder.iter().enumerate() {
            let byte_idx = data.len() - remainder.len() + idx;
            encode_byte_scalar(byte, byte_idx, &mut output, &alphabets);
        }
        
        output
    }
    
    /// Encode a single byte using scalar operations
    fn encode_byte_scalar(byte: u8, byte_idx: usize, output: &mut String, alphabets: &[[u8; 16]; 4]) {
        let hi_nibble = (byte >> 4) & 0xF;
        let lo_nibble = byte & 0xF;
        let base_pos = byte_idx * 2;
        
        output.push(alphabets[base_pos & 3][hi_nibble as usize] as char);
        output.push(alphabets[(base_pos + 1) & 3][lo_nibble as usize] as char);
    }
    
    /// Get Q64 alphabet tables
    fn get_q64_alphabets() -> [[u8; 16]; 4] {
        // This must match the alphabets from the original Q64 implementation
        [
            *b"ABCDEFGHIJKLMNOP", // Alphabet 0: pos ≡ 0 (mod 4)
            *b"QRSTUVWXYZabcdef", // Alphabet 1: pos ≡ 1 (mod 4)
            *b"ghijklmnopqrstuv", // Alphabet 2: pos ≡ 2 (mod 4)
            *b"wxyz0123456789-_", // Alphabet 3: pos ≡ 3 (mod 4)
        ]
    }
}

/// SIMD-optimized Top-k operations
pub mod topk_simd {
    use super::dispatch::SimdLevel;
    
    /// Find maximum values using SIMD
    pub fn find_max_indices_simd_dispatch(data: &[u8], k: usize) -> Vec<usize> {
        let simd_level = super::dispatch::detect_simd_level();
        
        match simd_level {
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Avx2 => unsafe { find_max_indices_avx2(data, k) },
            #[cfg(target_arch = "x86_64")]
            SimdLevel::Sse2 | SimdLevel::Sse41 => unsafe { find_max_indices_sse2(data, k) },
            #[cfg(target_arch = "aarch64")]
            SimdLevel::Neon => unsafe { find_max_indices_neon(data, k) },
            _ => find_max_indices_scalar(data, k),
        }
    }
    
    /// Scalar fallback for finding max indices
    pub fn find_max_indices_scalar(data: &[u8], k: usize) -> Vec<usize> {
        let mut indexed: Vec<(u8, usize)> = data
            .iter()
            .enumerate()
            .map(|(idx, &val)| (val, idx))
            .collect();
        
        let k_clamped = k.min(indexed.len());
        if k_clamped > 0 {
            indexed.select_nth_unstable_by(k_clamped - 1, |a, b| b.0.cmp(&a.0));
        }
        
        indexed[..k_clamped].iter().map(|(_, idx)| *idx).collect()
    }
    
    /// AVX2 implementation for finding max indices
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn find_max_indices_avx2(data: &[u8], k: usize) -> Vec<usize> {
        #[cfg(target_arch = "x86_64")]
        #[allow(unused_imports)]
        use std::arch::x86_64::*;
        
        if k == 1 {
            // Optimize for k=1 (finding single maximum)
            return vec![find_single_max_avx2(data)];
        }
        
        // For k > 1, fall back to scalar for now
        // A full SIMD implementation would require sorting networks
        find_max_indices_scalar(data, k)
    }
    
    /// Find single maximum using AVX2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn find_single_max_avx2(data: &[u8]) -> usize {
        #[cfg(target_arch = "x86_64")]
        #[allow(unused_imports)]
        use std::arch::x86_64::*;
        
        if data.len() < 32 {
            return find_max_indices_scalar(data, 1)[0];
        }
        
        let mut max_val = 0u8;
        let mut max_idx = 0usize;
        
        // Process 32 bytes at a time
        let chunks = data.chunks_exact(32);
        let remainder = chunks.remainder();
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            let chunk_data = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            
            // Find max within this chunk
            let chunk_max = find_max_in_vector_avx2(chunk_data);
            
            // Compare with global max
            let comparison = _mm256_cmpeq_epi8(chunk_data, _mm256_set1_epi8(chunk_max as i8));
            
            if chunk_max > max_val {
                max_val = chunk_max;
                // Find first occurrence of max value in this chunk
                let mask = _mm256_movemask_epi8(comparison);
                if mask != 0 {
                    let first_bit = mask.trailing_zeros() as usize;
                    max_idx = chunk_idx * 32 + first_bit;
                }
            }
        }
        
        // Process remainder
        for (i, &val) in remainder.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = data.len() - remainder.len() + i;
            }
        }
        
        max_idx
    }
    
    /// Find maximum value in a 256-bit vector using AVX2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn find_max_in_vector_avx2(vec: std::arch::x86_64::__m256i) -> u8 {
        use std::arch::x86_64::*;
        
        // Horizontal maximum reduction
        let max_lo = _mm256_extracti128_si256(vec, 0);
        let max_hi = _mm256_extracti128_si256(vec, 1);
        let max_128 = _mm_max_epu8(max_lo, max_hi);
        
        // Continue reduction in 128-bit register
        let max_64 = _mm_max_epu8(max_128, _mm_srli_si128(max_128, 8));
        let max_32 = _mm_max_epu8(max_64, _mm_srli_si128(max_64, 4));
        let max_16 = _mm_max_epu8(max_32, _mm_srli_si128(max_32, 2));
        let max_8 = _mm_max_epu8(max_16, _mm_srli_si128(max_16, 1));
        
        _mm_extract_epi8(max_8, 0) as u8
    }
    
    /// SSE2 implementation for finding max indices
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn find_max_indices_sse2(data: &[u8], k: usize) -> Vec<usize> {
        if k == 1 {
            return vec![find_single_max_sse2(data)];
        }
        find_max_indices_scalar(data, k)
    }
    
    /// Find single maximum using SSE2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn find_single_max_sse2(data: &[u8]) -> usize {
        #[cfg(target_arch = "x86_64")]
        #[allow(unused_imports)]
        use std::arch::x86_64::*;
        
        if data.len() < 16 {
            return find_max_indices_scalar(data, 1)[0];
        }
        
        let mut max_val = 0u8;
        let mut max_idx = 0usize;
        
        // Process 16 bytes at a time
        let chunks = data.chunks_exact(16);
        let remainder = chunks.remainder();
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            let chunk_data = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            
            // Find max within this chunk using horizontal reduction
            let chunk_max = find_max_in_vector_sse2(chunk_data);
            
            if chunk_max > max_val {
                max_val = chunk_max;
                // Find first occurrence of max value in this chunk
                let comparison = _mm_cmpeq_epi8(chunk_data, _mm_set1_epi8(chunk_max as i8));
                let mask = _mm_movemask_epi8(comparison);
                if mask != 0 {
                    let first_bit = mask.trailing_zeros() as usize;
                    max_idx = chunk_idx * 16 + first_bit;
                }
            }
        }
        
        // Process remainder
        for (i, &val) in remainder.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = data.len() - remainder.len() + i;
            }
        }
        
        max_idx
    }
    
    /// Find maximum value in a 128-bit vector using SSE2
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn find_max_in_vector_sse2(vec: std::arch::x86_64::__m128i) -> u8 {
        use std::arch::x86_64::*;
        
        // Horizontal maximum reduction for unsigned bytes
        let max_64 = _mm_max_epu8(vec, _mm_srli_si128(vec, 8));
        let max_32 = _mm_max_epu8(max_64, _mm_srli_si128(max_64, 4));
        let max_16 = _mm_max_epu8(max_32, _mm_srli_si128(max_32, 2));
        let max_8 = _mm_max_epu8(max_16, _mm_srli_si128(max_16, 1));
        
        _mm_extract_epi16(max_8, 0) as u8 & 0xFF
    }
    
    /// NEON implementation for finding max indices
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn find_max_indices_neon(data: &[u8], k: usize) -> Vec<usize> {
        if k == 1 {
            return vec![find_single_max_neon(data)];
        }
        find_max_indices_scalar(data, k)
    }
    
    /// Find single maximum using NEON
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn find_single_max_neon(data: &[u8]) -> usize {
        // For now, fall back to scalar implementation to ensure correctness
        find_max_indices_scalar(data, 1)[0]
    }
}

/// SIMD benchmark utilities
pub mod benchmark {
    use super::dispatch::{detect_simd_level, simd_level_name};
    use std::time::Instant;
    
    /// Benchmark SIMD implementations
    pub fn benchmark_simd_implementations() {
        println!("SIMD Capability Detection:");
        let level = detect_simd_level();
        println!("Detected SIMD level: {}", simd_level_name(level));
        
        // Test data
        let test_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        
        println!("\nQ64 Encoding Benchmark:");
        
        // Scalar benchmark
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = crate::encoders::q64_encode(&test_data);
        }
        let scalar_time = start.elapsed();
        println!("Scalar: {:?}", scalar_time);
        
        // SIMD benchmark
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = super::q64_simd::q64_encode_simd_dispatch(&test_data);
        }
        let simd_time = start.elapsed();
        println!("SIMD: {:?}", simd_time);
        
        if simd_time < scalar_time {
            let speedup = scalar_time.as_nanos() as f64 / simd_time.as_nanos() as f64;
            println!("SIMD speedup: {:.2}x", speedup);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simd_detection() {
        let level = dispatch::detect_simd_level();
        println!("Detected SIMD level: {}", dispatch::simd_level_name(level));
        // Just ensure detection doesn't crash
        assert!(matches!(level, dispatch::SimdLevel::Scalar | 
                              dispatch::SimdLevel::Sse2 | 
                              dispatch::SimdLevel::Sse41 | 
                              dispatch::SimdLevel::Avx2 | 
                              dispatch::SimdLevel::Avx512 | 
                              dispatch::SimdLevel::Neon));
    }
    
    #[test]
    fn test_simd_q64_consistency() {
        let test_data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        
        let scalar_result = crate::encoders::q64_encode(&test_data);
        let simd_result = q64_simd::q64_encode_simd_dispatch(&test_data);
        
        assert_eq!(scalar_result, simd_result);
    }
    
    #[test]
    fn test_simd_topk_consistency() {
        let test_data: Vec<u8> = (0..100).map(|i| (i * 13 % 256) as u8).collect();
        
        let scalar_result = topk_simd::find_max_indices_scalar(&test_data, 1);
        let simd_result = topk_simd::find_max_indices_simd_dispatch(&test_data, 1);
        
        assert_eq!(scalar_result, simd_result);
    }
}