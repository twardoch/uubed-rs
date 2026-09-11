# this_file: tests/test_native.py
"""Built extension checks: no import skips or placeholder success tests."""
import json

import numpy as np
import pytest
import uubed_native as native


def test_q64_when_all_bytes_then_lossless_roundtrip():
    data = bytes(range(256))
    encoded = native.q64_encode_native(data)
    assert bytes(native.q64_decode_native(encoded)) == data
    with pytest.raises(ValueError):
        native.q64_decode_native("!")


def test_numpy_when_buffers_then_same_encoding_and_batch_order():
    data = np.arange(256, dtype=np.uint8)
    expected = native.q64_encode_native(data.tobytes()).encode()
    assert native.q64_encode_buffer_native(data) == expected
    output = np.zeros(512, dtype=np.uint8)
    assert native.q64_encode_inplace_native(data, output) == len(expected)
    assert output.tobytes() == expected
    assert native.q64_encode_batch_native([data, data[::-1].copy()], True) == [
        expected, native.q64_encode_native(data[::-1].tobytes()).encode()]


def test_memory_when_saved_then_reopen_without_extension_dependencies(tmp_path):
    path = tmp_path / "memory.sqlite"
    store = native.NativeTm.create(str(path), '{"format":"test"}', 2)
    store.add_pairs(json.dumps([dict(source="font", target="krój", language="pl", origin="fixture", unit="1")]))
    store.set_vectors([(1, [127, 0], 1 / 127)])
    store.close()
    read = native.NativeTm(str(path))
    assert json.loads(read.exact("font", "pl"))[0]["target"] == "krój"
    assert json.loads(read.search([1., 0.], "pl", 1, .5))[0]["score"] == 1
    read.close()
    with pytest.raises(ValueError, match="closed"):
        read.stats()
