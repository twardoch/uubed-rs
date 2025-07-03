import pytest


def test_import():
    try:
        import uubed_rs
        assert uubed_rs is not None
    except ImportError:
        pytest.skip("uubed_rs module not installed")


def test_placeholder():
    assert True