import pytest
import numpy as np

try:
    import uubed_rs
except ImportError:
    pytest.skip("uubed_rs module not installed", allow_module_level=True)


class TestNumpyIntegration:
    def test_q64_encode_numpy_array(self):
        # Test encoding of numpy arrays
        data = np.random.randint(0, 256, size=1024, dtype=np.uint8)
        
        # Should work directly with numpy arrays
        encoded = uubed_rs.q64_encode_buffer_native(data)
        assert isinstance(encoded, bytes)
        assert len(encoded) == len(data) * 2
        
    def test_q64_batch_encode_numpy(self):
        # Test batch encoding with numpy arrays
        batch_size = 100
        embedding_size = 384
        
        embeddings = [
            np.random.randint(0, 256, size=embedding_size, dtype=np.uint8) 
            for _ in range(batch_size)
        ]
        
        # Test with buffer reuse
        results = uubed_rs.q64_encode_batch_native(embeddings, reuse_buffers=True)
        assert len(results) == batch_size
        assert all(isinstance(r, bytes) for r in results)
        assert all(len(r) == embedding_size * 2 for r in results)
        
    def test_q64_inplace_numpy(self):
        # Test in-place encoding with pre-allocated numpy buffer
        input_data = np.random.randint(0, 256, size=512, dtype=np.uint8)
        output_buffer = np.zeros(1024, dtype=np.uint8)  # 2x size for Q64
        
        bytes_written = uubed_rs.q64_encode_inplace_native(input_data, output_buffer)
        assert bytes_written == 1024
        
        # Verify output buffer was modified
        assert not np.all(output_buffer == 0)
        
    def test_simhash_numpy_buffer(self):
        # Test SimHash with numpy arrays
        embedding = np.random.randint(0, 256, size=1536, dtype=np.uint8)
        planes = 64
        output_size = planes // 4  # 64 bits = 16 Q64 chars
        output_buffer = np.zeros(output_size, dtype=np.uint8)
        
        bytes_written = uubed_rs.simhash_to_buffer_native(embedding, planes, output_buffer)
        assert bytes_written == output_size
        
    def test_topk_numpy_buffer(self):
        # Test Top-K with numpy arrays
        embedding = np.random.randint(0, 256, size=768, dtype=np.uint8)
        k = 10
        output_size = k * 2  # Each index encoded as 2 Q64 chars
        output_buffer = np.zeros(output_size, dtype=np.uint8)
        
        bytes_written = uubed_rs.top_k_to_buffer_native(embedding, k, output_buffer)
        assert bytes_written == output_size
        
    def test_parallel_numpy_batch(self):
        # Test parallel encoding with numpy arrays
        batch_size = 1000
        embedding_size = 256
        
        embeddings = [
            np.random.randint(0, 256, size=embedding_size, dtype=np.uint8) 
            for _ in range(batch_size)
        ]
        
        # Test parallel Q64 encoding
        results = uubed_rs.parallel_q64_encode_native(embeddings, num_threads=4)
        assert len(results) == batch_size
        assert all(isinstance(r, str) for r in results)
        
        # Test parallel SimHash encoding
        simhash_results = uubed_rs.parallel_simhash_encode_native(embeddings, 64, num_threads=4)
        assert len(simhash_results) == batch_size
        
        # Test parallel Top-K encoding
        topk_results = uubed_rs.parallel_topk_encode_native(embeddings, 8, num_threads=4)
        assert len(topk_results) == batch_size
        
    def test_stream_encoder_numpy(self):
        # Test streaming encoder with numpy chunks
        encoder = uubed_rs.Q64StreamEncoder(chunk_size=65536)
        
        # Process multiple chunks
        chunk_results = []
        for _ in range(10):
            chunk = np.random.randint(0, 256, size=8192, dtype=np.uint8)
            encoded = encoder.encode_chunk(chunk)
            chunk_results.append(encoded)
            
        assert len(chunk_results) == 10
        assert all(isinstance(r, bytes) for r in chunk_results)
        assert all(len(r) == 16384 for r in chunk_results)  # 8192 * 2
        
    def test_buffer_pool_numpy(self):
        # Test buffer pool with numpy operations
        pool = uubed_rs.BufferPool(max_pool_size=50)
        
        # Simulate batch processing with buffer reuse
        batch_processor = uubed_rs.SimpleBatchProcessor(chunk_size=100)
        
        embeddings = [
            np.random.randint(0, 256, size=128, dtype=np.uint8) 
            for _ in range(500)
        ]
        
        results = batch_processor.process_batch(embeddings)
        assert len(results) == 500
        
        # Check pool stats
        stats = pool.get_stats()
        assert 'allocations' in stats
        assert 'reuses' in stats


class TestMemoryEfficiency:
    def test_large_batch_memory(self):
        # Test memory efficiency with large batches
        batch_size = 10000
        embedding_size = 384
        
        # Create a view into a large numpy array to minimize copies
        large_array = np.random.randint(0, 256, size=(batch_size, embedding_size), dtype=np.uint8)
        
        # Process using views
        embeddings = [large_array[i] for i in range(batch_size)]
        
        # Use batch processor for memory-efficient processing
        processor = uubed_rs.SimpleBatchProcessor(chunk_size=1000)
        results = processor.process_batch(embeddings)
        
        assert len(results) == batch_size
        
    def test_zero_copy_roundtrip(self):
        # Test zero-copy operations
        original = np.random.randint(0, 256, size=1024, dtype=np.uint8)
        
        # Encode using buffer API
        encoded = uubed_rs.q64_encode_buffer_native(original)
        
        # Decode back
        decoded = uubed_rs.q64_decode_native(encoded.decode('ascii'))
        decoded_array = np.array(decoded, dtype=np.uint8)
        
        # Verify roundtrip
        np.testing.assert_array_equal(original, decoded_array)