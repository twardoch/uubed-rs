#!/usr/bin/env python3
"""
Example usage of the enhanced uubed-native Python bindings

This demonstrates the advanced PyO3 functionality including:
- PyBuffer support for numpy arrays
- Batch processing with buffer pooling
- Streaming encoders for large data
- Performance monitoring
"""

import numpy as np
import time
from typing import List

# Note: This would normally be imported after building the library
# import uubed_native

def demo_basic_encoding():
    """Demonstrate basic Q64 encoding functionality"""
    print("=== Basic Q64 Encoding Demo ===")
    
    # Test data
    data = bytes([0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0])
    print(f"Original data: {data.hex()}")
    
    # Basic encoding
    # encoded = uubed_native.q64_encode_native(data)
    # print(f"Q64 encoded: {encoded}")
    
    # Decoding
    # decoded = uubed_native.q64_decode_native(encoded)
    # print(f"Decoded: {decoded.hex()}")
    # print(f"Roundtrip successful: {data == bytes(decoded)}")
    print("Basic encoding would work here with compiled library")

def demo_buffer_encoding():
    """Demonstrate PyBuffer support with numpy arrays"""
    print("\n=== PyBuffer Support Demo ===")
    
    # Create numpy array (this would be supported by PyBuffer)
    data = np.array([0x12, 0x34, 0x56, 0x78], dtype=np.uint8)
    print(f"Numpy array: {data}")
    
    # Buffer encoding (zero-copy from numpy)
    # encoded = uubed_native.q64_encode_buffer_native(data)
    # print(f"Buffer encoded: {encoded}")
    print("Buffer encoding would work with numpy arrays")

def demo_batch_processing():
    """Demonstrate batch processing with buffer pooling"""
    print("\n=== Batch Processing Demo ===")
    
    # Create multiple embeddings
    embeddings = [
        np.random.randint(0, 256, size=1000, dtype=np.uint8) 
        for _ in range(100)
    ]
    print(f"Created {len(embeddings)} embeddings of size 1000 each")
    
    # Batch processing with buffer reuse
    # start_time = time.time()
    # results = uubed_native.q64_encode_batch_native(embeddings, reuse_buffers=True)
    # batch_time = time.time() - start_time
    # print(f"Batch encoding completed in {batch_time:.3f}s")
    # print(f"Average time per embedding: {batch_time/len(embeddings)*1000:.2f}ms")
    print("Batch processing would provide significant speedup")

def demo_streaming_encoder():
    """Demonstrate streaming encoder for very large data"""
    print("\n=== Streaming Encoder Demo ===")
    
    # Create streaming encoder
    # encoder = uubed_native.Q64StreamEncoder(chunk_size=8192)
    # print(f"Created streaming encoder with chunk size: {encoder.get_chunk_size()}")
    
    # Large data simulation
    chunk_size = 8192
    num_chunks = 10
    total_size = chunk_size * num_chunks
    print(f"Processing {total_size} bytes in {num_chunks} chunks")
    
    # Process chunks
    # for i in range(num_chunks):
    #     chunk = np.random.randint(0, 256, size=chunk_size, dtype=np.uint8)
    #     encoded_chunk = encoder.encode_chunk(chunk)
    #     print(f"Processed chunk {i+1}/{num_chunks}: {len(encoded_chunk)} bytes")
    print("Streaming encoder would handle very large datasets efficiently")

def demo_performance_monitoring():
    """Demonstrate performance monitoring and statistics"""
    print("\n=== Performance Monitoring Demo ===")
    
    # Create statistics tracker
    # stats = uubed_native.Q64Stats()
    
    # Simulate operations (this would be tracked automatically)
    operations = [
        (1000, "Small embedding"),
        (10000, "Medium embedding"), 
        (100000, "Large embedding")
    ]
    
    for size, description in operations:
        # data = np.random.randint(0, 256, size=size, dtype=np.uint8)
        # encoded = uubed_native.q64_encode_buffer_native(data)
        print(f"Processed {description}: {size} bytes")
    
    # Get statistics
    # performance_stats = stats.get_stats()
    # print("Performance Statistics:")
    # for key, value in performance_stats.items():
    #     print(f"  {key}: {value}")
    print("Performance monitoring would track all operations")

def demo_buffer_pool():
    """Demonstrate memory pool for efficient buffer reuse"""
    print("\n=== Buffer Pool Demo ===")
    
    # Create buffer pool
    # pool = uubed_native.BufferPool(max_pool_size=50)
    
    # Simulate buffer usage patterns
    sizes = [1000, 5000, 10000, 1000, 5000]  # Some repeating sizes
    
    for i, size in enumerate(sizes):
        # buffer = pool.get_buffer(size)
        # # Use buffer...
        # pool.return_buffer(buffer)
        print(f"Operation {i+1}: Used {size}-byte buffer")
    
    # Get pool statistics
    # pool_stats = pool.get_stats()
    # print("Buffer Pool Statistics:")
    # for key, value in pool_stats.items():
    #     print(f"  {key}: {value}")
    print("Buffer pool would show high reuse efficiency")

def demo_simple_batch_processor():
    """Demonstrate simplified batch processor"""
    print("\n=== Simple Batch Processor Demo ===")
    
    # Create batch processor
    # processor = uubed_native.SimpleBatchProcessor(chunk_size=1000)
    # print(f"Created batch processor with chunk size: {processor.get_chunk_size()}")
    
    # Large batch of embeddings
    batch_size = 5000
    embedding_size = 512
    
    # embeddings = [
    #     np.random.randint(0, 256, size=embedding_size, dtype=np.uint8)
    #     for _ in range(batch_size)
    # ]
    
    # start_time = time.time()
    # results = processor.process_batch(embeddings)
    # process_time = time.time() - start_time
    
    # print(f"Processed {batch_size} embeddings in {process_time:.3f}s")
    # print(f"Throughput: {batch_size/process_time:.0f} embeddings/second")
    print(f"Would process {batch_size} embeddings with automatic chunking")

def main():
    """Run all demonstrations"""
    print("uubed-native Enhanced PyO3 Bindings Demo")
    print("=" * 50)
    
    demo_basic_encoding()
    demo_buffer_encoding()
    demo_batch_processing()
    demo_streaming_encoder()
    demo_performance_monitoring()
    demo_buffer_pool()
    demo_simple_batch_processor()
    
    print("\n" + "=" * 50)
    print("Demo completed! Build the library to see actual functionality.")
    print("\nKey features demonstrated:")
    print("- PyBuffer support for numpy arrays (zero-copy from Python)")
    print("- Batch processing with buffer pooling")
    print("- Streaming encoders for large datasets")
    print("- Performance monitoring and statistics")
    print("- Memory pool for efficient buffer reuse")
    print("- Simplified batch processors with automatic chunking")

if __name__ == "__main__":
    main()