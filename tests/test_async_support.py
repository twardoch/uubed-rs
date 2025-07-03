import pytest
import asyncio
import numpy as np
from concurrent.futures import ThreadPoolExecutor

try:
    import uubed_rs
except ImportError:
    pytest.skip("uubed_rs module not installed", allow_module_level=True)


class TestAsyncSupport:
    @pytest.mark.asyncio
    async def test_async_batch_processing(self):
        # Test async batch processing with thread pool
        executor = ThreadPoolExecutor(max_workers=4)
        
        async def encode_batch_async(embeddings):
            loop = asyncio.get_event_loop()
            return await loop.run_in_executor(
                executor, 
                uubed_rs.parallel_q64_encode_native,
                embeddings,
                4
            )
        
        # Create test data
        batch_size = 1000
        embedding_size = 256
        embeddings = [
            np.random.randint(0, 256, size=embedding_size, dtype=np.uint8) 
            for _ in range(batch_size)
        ]
        
        # Process asynchronously
        results = await encode_batch_async(embeddings)
        assert len(results) == batch_size
        
        executor.shutdown()
        
    @pytest.mark.asyncio
    async def test_concurrent_encoding(self):
        # Test concurrent encoding of multiple batches
        async def encode_batch(batch_id, size):
            embeddings = [
                np.random.randint(0, 256, size=128, dtype=np.uint8) 
                for _ in range(size)
            ]
            
            loop = asyncio.get_event_loop()
            results = await loop.run_in_executor(
                None,
                uubed_rs.q64_encode_batch_native,
                embeddings,
                True
            )
            return batch_id, len(results)
        
        # Launch multiple concurrent batches
        tasks = [
            encode_batch(i, 100) for i in range(10)
        ]
        
        results = await asyncio.gather(*tasks)
        
        # Verify all batches completed
        assert len(results) == 10
        assert all(count == 100 for _, count in results)
        
    @pytest.mark.asyncio
    async def test_streaming_async(self):
        # Test async streaming with progress updates
        class AsyncStreamProcessor:
            def __init__(self):
                self.encoder = uubed_rs.Q64StreamEncoder(chunk_size=8192)
                self.processed = 0
                
            async def process_stream(self, chunks):
                results = []
                for i, chunk in enumerate(chunks):
                    # Simulate async I/O or computation
                    await asyncio.sleep(0.001)
                    
                    # Encode chunk
                    loop = asyncio.get_event_loop()
                    encoded = await loop.run_in_executor(
                        None,
                        self.encoder.encode_chunk,
                        chunk
                    )
                    results.append(encoded)
                    self.processed += 1
                    
                    # Could yield progress here
                    if i % 10 == 0:
                        print(f"Processed {i+1}/{len(chunks)} chunks")
                        
                return results
        
        # Create test chunks
        chunks = [
            np.random.randint(0, 256, size=1024, dtype=np.uint8)
            for _ in range(50)
        ]
        
        processor = AsyncStreamProcessor()
        results = await processor.process_stream(chunks)
        
        assert len(results) == 50
        assert processor.processed == 50
        
    @pytest.mark.asyncio
    async def test_async_with_timeout(self):
        # Test async operations with timeout
        async def slow_encoding():
            # Simulate a large batch that takes time
            embeddings = [
                np.random.randint(0, 256, size=1024, dtype=np.uint8) 
                for _ in range(5000)
            ]
            
            loop = asyncio.get_event_loop()
            return await loop.run_in_executor(
                None,
                uubed_rs.parallel_q64_encode_native,
                embeddings,
                1  # Use single thread to make it slower
            )
        
        # Should complete within timeout
        try:
            results = await asyncio.wait_for(slow_encoding(), timeout=10.0)
            assert len(results) == 5000
        except asyncio.TimeoutError:
            pytest.fail("Encoding timed out")
            
    @pytest.mark.asyncio
    async def test_async_pipeline(self):
        # Test async pipeline with multiple stages
        async def pipeline():
            # Stage 1: Generate embeddings
            embeddings = await generate_embeddings_async(100)
            
            # Stage 2: Encode with Q64
            q64_encoded = await encode_async(embeddings, 'q64')
            
            # Stage 3: Generate SimHash
            simhash_encoded = await encode_async(embeddings, 'simhash')
            
            # Stage 4: Generate Top-K
            topk_encoded = await encode_async(embeddings, 'topk')
            
            return q64_encoded, simhash_encoded, topk_encoded
        
        async def generate_embeddings_async(count):
            await asyncio.sleep(0.01)  # Simulate async generation
            return [
                np.random.randint(0, 256, size=256, dtype=np.uint8)
                for _ in range(count)
            ]
        
        async def encode_async(embeddings, method):
            loop = asyncio.get_event_loop()
            
            if method == 'q64':
                return await loop.run_in_executor(
                    None,
                    uubed_rs.parallel_q64_encode_native,
                    embeddings,
                    4
                )
            elif method == 'simhash':
                return await loop.run_in_executor(
                    None,
                    uubed_rs.parallel_simhash_encode_native,
                    embeddings,
                    64,
                    4
                )
            elif method == 'topk':
                return await loop.run_in_executor(
                    None,
                    uubed_rs.parallel_topk_encode_native,
                    embeddings,
                    8,
                    4
                )
        
        q64, simhash, topk = await pipeline()
        assert len(q64) == 100
        assert len(simhash) == 100
        assert len(topk) == 100


class TestAsyncProgressCallbacks:
    @pytest.mark.asyncio
    async def test_progress_callback_simulation(self):
        # Simulate progress callbacks for long-running operations
        class ProgressTracker:
            def __init__(self):
                self.progress_queue = asyncio.Queue()
                self.total_items = 0
                self.processed_items = 0
                
            async def process_with_progress(self, embeddings, chunk_size=100):
                self.total_items = len(embeddings)
                results = []
                
                for i in range(0, len(embeddings), chunk_size):
                    chunk = embeddings[i:i+chunk_size]
                    
                    # Process chunk
                    loop = asyncio.get_event_loop()
                    chunk_results = await loop.run_in_executor(
                        None,
                        uubed_rs.q64_encode_batch_native,
                        chunk,
                        True
                    )
                    results.extend(chunk_results)
                    
                    # Update progress
                    self.processed_items += len(chunk)
                    progress = self.processed_items / self.total_items
                    await self.progress_queue.put(progress)
                    
                return results
            
            async def monitor_progress(self):
                progress_values = []
                while True:
                    try:
                        progress = await asyncio.wait_for(
                            self.progress_queue.get(), 
                            timeout=0.1
                        )
                        progress_values.append(progress)
                        if progress >= 1.0:
                            break
                    except asyncio.TimeoutError:
                        break
                return progress_values
        
        # Create test data
        embeddings = [
            np.random.randint(0, 256, size=128, dtype=np.uint8)
            for _ in range(1000)
        ]
        
        tracker = ProgressTracker()
        
        # Run processing and monitoring concurrently
        process_task = asyncio.create_task(
            tracker.process_with_progress(embeddings)
        )
        monitor_task = asyncio.create_task(
            tracker.monitor_progress()
        )
        
        results, progress_values = await asyncio.gather(
            process_task, monitor_task
        )
        
        assert len(results) == 1000
        assert len(progress_values) > 0
        assert progress_values[-1] == 1.0