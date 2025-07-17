# this_file: tests/test_release_workflow.py
"""
Release workflow tests for uubed-rs
Tests that verify the library works correctly after installation
"""

import sys
import importlib.util
import pytest
import numpy as np


def test_library_imports():
    """Test that the library can be imported successfully"""
    try:
        import uubed
        assert hasattr(uubed, 'q64_encode')
        assert hasattr(uubed, 'q64_decode')
        assert hasattr(uubed, 'mq64_encode')
        assert hasattr(uubed, 'mq64_decode')
    except ImportError as e:
        pytest.skip(f"uubed library not available: {e}")


def test_version_information():
    """Test that version information is available"""
    try:
        import uubed
        
        # Check if version is available
        if hasattr(uubed, '__version__'):
            version = uubed.__version__
            assert isinstance(version, str)
            assert len(version) > 0
            
            # Check semver format
            parts = version.split('.')
            assert len(parts) >= 3, f"Version should have at least 3 parts, got: {version}"
            
            # Check that major, minor, patch are numbers
            for i, part in enumerate(parts[:3]):
                assert part.isdigit(), f"Version part {i} should be numeric, got: {part}"
    except ImportError:
        pytest.skip("uubed library not available")


def test_basic_functionality():
    """Test basic encoding/decoding functionality"""
    try:
        import uubed
        
        # Test with simple data
        test_data = [1.0, 2.0, 3.0, 4.0]
        
        # Test Q64 encoding
        encoded = uubed.q64_encode(test_data)
        assert isinstance(encoded, str)
        assert len(encoded) > 0
        
        # Test Q64 decoding
        decoded = uubed.q64_decode(encoded)
        assert isinstance(decoded, list)
        assert len(decoded) == len(test_data)
        
        # Test MQ64 encoding
        mq64_encoded = uubed.mq64_encode(test_data)
        assert isinstance(mq64_encoded, str)
        assert len(mq64_encoded) > 0
        
        # Test MQ64 decoding
        mq64_decoded = uubed.mq64_decode(mq64_encoded)
        assert isinstance(mq64_decoded, list)
        assert len(mq64_decoded) == len(test_data)
        
    except ImportError:
        pytest.skip("uubed library not available")


def test_numpy_integration():
    """Test that numpy integration works properly"""
    try:
        import uubed
        
        # Test with numpy array
        test_array = np.array([1.0, 2.0, 3.0, 4.0], dtype=np.float32)
        
        # Test Q64 with numpy
        encoded = uubed.q64_encode(test_array.tolist())
        assert isinstance(encoded, str)
        
        decoded = uubed.q64_decode(encoded)
        assert isinstance(decoded, list)
        
        # Convert back to numpy and compare
        decoded_array = np.array(decoded, dtype=np.float32)
        assert decoded_array.shape == test_array.shape
        
    except ImportError:
        pytest.skip("uubed library not available")


def test_error_handling():
    """Test that error handling works properly"""
    try:
        import uubed
        
        # Test with invalid input
        with pytest.raises(Exception):
            uubed.q64_encode(None)
        
        # Test with empty string decode
        with pytest.raises(Exception):
            uubed.q64_decode("")
        
        # Test with invalid string decode
        with pytest.raises(Exception):
            uubed.q64_decode("invalid_base64")
            
    except ImportError:
        pytest.skip("uubed library not available")


def test_performance_characteristics():
    """Test that performance characteristics are reasonable"""
    try:
        import uubed
        import time
        
        # Test with larger dataset
        large_data = list(range(1000))
        large_data = [float(x) for x in large_data]
        
        # Measure encoding time
        start_time = time.time()
        encoded = uubed.q64_encode(large_data)
        encode_time = time.time() - start_time
        
        # Measure decoding time
        start_time = time.time()
        decoded = uubed.q64_decode(encoded)
        decode_time = time.time() - start_time
        
        # Performance should be reasonable (less than 1 second for 1000 elements)
        assert encode_time < 1.0, f"Encoding took too long: {encode_time}s"
        assert decode_time < 1.0, f"Decoding took too long: {decode_time}s"
        
        # Check correctness
        assert len(decoded) == len(large_data)
        
    except ImportError:
        pytest.skip("uubed library not available")


def test_memory_usage():
    """Test that memory usage is reasonable"""
    try:
        import uubed
        import gc
        
        # Test that we don't have major memory leaks
        initial_objects = len(gc.get_objects())
        
        # Perform many operations
        for i in range(100):
            data = [float(j) for j in range(10)]
            encoded = uubed.q64_encode(data)
            decoded = uubed.q64_decode(encoded)
        
        # Force garbage collection
        gc.collect()
        
        final_objects = len(gc.get_objects())
        
        # We shouldn't have created too many persistent objects
        object_increase = final_objects - initial_objects
        assert object_increase < 1000, f"Too many objects created: {object_increase}"
        
    except ImportError:
        pytest.skip("uubed library not available")


def test_multiprocessing_compatibility():
    """Test that the library works with multiprocessing"""
    try:
        import uubed
        import multiprocessing as mp
        
        def worker(data):
            encoded = uubed.q64_encode(data)
            decoded = uubed.q64_decode(encoded)
            return len(decoded)
        
        # Test with a simple multiprocessing scenario
        if mp.get_start_method() == 'fork' or sys.platform != 'darwin':
            test_data = [1.0, 2.0, 3.0, 4.0]
            
            with mp.Pool(processes=2) as pool:
                results = pool.map(worker, [test_data, test_data])
                
            assert all(result == len(test_data) for result in results)
        
    except ImportError:
        pytest.skip("uubed library not available")


def test_thread_safety():
    """Test that the library is thread-safe"""
    try:
        import uubed
        import threading
        import time
        
        results = []
        errors = []
        
        def worker():
            try:
                for i in range(10):
                    data = [float(j) for j in range(10)]
                    encoded = uubed.q64_encode(data)
                    decoded = uubed.q64_decode(encoded)
                    results.append(len(decoded))
                    time.sleep(0.001)  # Small delay to increase chance of race conditions
            except Exception as e:
                errors.append(e)
        
        # Start multiple threads
        threads = []
        for _ in range(5):
            t = threading.Thread(target=worker)
            threads.append(t)
            t.start()
        
        # Wait for all threads to complete
        for t in threads:
            t.join()
        
        # Check results
        assert len(errors) == 0, f"Thread safety errors: {errors}"
        assert len(results) == 50, f"Expected 50 results, got {len(results)}"
        assert all(result == 10 for result in results), "All results should be 10"
        
    except ImportError:
        pytest.skip("uubed library not available")


if __name__ == "__main__":
    # Run tests directly
    pytest.main([__file__, "-v"])