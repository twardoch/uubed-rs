// this_file: rust/examples/zero_copy_demo.rs
//! Demonstration of zero-copy Q64 encoding performance benefits

use std::time::Instant;
use uubed_native::encoders::{q64_encode, q64_encode_to_buffer};

fn main() {
    println!("Zero-Copy Q64 Encoding Performance Demo");
    println!("======================================");
    
    // Test with different data sizes
    let sizes = vec![1000, 10000, 100000];
    
    for size in sizes {
        println!("\nTesting with {} bytes:", size);
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        
        // Test string allocation version
        let start = Instant::now();
        let iterations = 1000;
        for _ in 0..iterations {
            let _result = q64_encode(&data);
        }
        let string_time = start.elapsed();
        
        // Test zero-copy version with buffer reuse
        let mut buffer = vec![0u8; data.len() * 2];
        let start = Instant::now();
        for _ in 0..iterations {
            let _bytes_written = q64_encode_to_buffer(&data, &mut buffer).unwrap();
        }
        let zero_copy_time = start.elapsed();
        
        // Calculate speedup
        let speedup = string_time.as_nanos() as f64 / zero_copy_time.as_nanos() as f64;
        
        println!("  String allocation: {:?}", string_time);
        println!("  Zero-copy:         {:?}", zero_copy_time);
        println!("  Speedup:           {:.2}x", speedup);
        
        // Memory allocation comparison
        println!("  Memory allocations:");
        println!("    String version:  {} allocations per call", 1);
        println!("    Zero-copy:       {} allocations per call", 0);
    }
    
    // Demonstrate correctness
    println!("\nCorrectness verification:");
    let test_data = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
    let string_result = q64_encode(&test_data);
    let mut buffer = vec![0u8; test_data.len() * 2];
    q64_encode_to_buffer(&test_data, &mut buffer).unwrap();
    let zero_copy_result = String::from_utf8(buffer).unwrap();
    
    println!("  String result:   {}", string_result);
    println!("  Zero-copy result: {}", zero_copy_result);
    println!("  Results match:   {}", string_result == zero_copy_result);
}