/**
 * @file c_api_demo.c
 * @brief Demonstration of the uubed C API
 * 
 * This example shows how to use the uubed library from C code.
 * It demonstrates:
 * - Q64 encoding and decoding
 * - Zero-copy operations
 * - SimHash encoding
 * - Top-K encoding
 * - Error handling
 * - Memory management
 * 
 * To compile:
 *   gcc -o c_api_demo c_api_demo.c -luubed_native -L../target/release
 * 
 * To run:
 *   LD_LIBRARY_PATH=../target/release ./c_api_demo
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <assert.h>

// Include the uubed header
#include "../include/uubed.h"

/**
 * @brief Print error message if operation failed
 */
static void check_error(UubedErrorCode code, const char* operation) {
    if (code != UUBED_SUCCESS) {
        const char* error_msg = uubed_get_last_error_message();
        fprintf(stderr, "Error in %s: %s (code: %d)\n", 
                operation, error_msg ? error_msg : "Unknown error", code);
        exit(1);
    }
}

/**
 * @brief Demonstrate basic Q64 encoding and decoding
 */
static void demo_q64_basic(void) {
    printf("=== Q64 Basic Encoding/Decoding ===\n");
    
    // Test data
    const uint8_t test_data[] = {0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF};
    const size_t test_len = sizeof(test_data);
    
    printf("Input data: ");
    for (size_t i = 0; i < test_len; i++) {
        printf("%02X ", test_data[i]);
    }
    printf("(%zu bytes)\n", test_len);
    
    // Encode
    char* encoded = NULL;
    UubedErrorCode result = uubed_q64_encode(test_data, test_len, &encoded);
    check_error(result, "Q64 encoding");
    
    printf("Encoded: %s (%zu chars)\n", encoded, strlen(encoded));
    
    // Decode
    uint8_t* decoded = NULL;
    size_t decoded_len = 0;
    result = uubed_q64_decode(encoded, &decoded, &decoded_len);
    check_error(result, "Q64 decoding");
    
    printf("Decoded: ");
    for (size_t i = 0; i < decoded_len; i++) {
        printf("%02X ", decoded[i]);
    }
    printf("(%zu bytes)\n", decoded_len);
    
    // Verify roundtrip
    assert(decoded_len == test_len);
    assert(memcmp(test_data, decoded, test_len) == 0);
    printf("✓ Roundtrip successful!\n\n");
    
    // Cleanup
    uubed_free_string(encoded);
    uubed_free_bytes(decoded, decoded_len);
}

/**
 * @brief Demonstrate zero-copy Q64 encoding
 */
static void demo_q64_zero_copy(void) {
    printf("=== Q64 Zero-Copy Encoding ===\n");
    
    const uint8_t test_data[] = {0xFF, 0x00, 0xAA, 0x55};
    const size_t test_len = sizeof(test_data);
    
    // Pre-allocate buffer (must be at least 2x input size)
    uint8_t buffer[test_len * 2];
    size_t bytes_written = 0;
    
    UubedErrorCode result = uubed_q64_encode_to_buffer(
        test_data, test_len, buffer, sizeof(buffer), &bytes_written
    );
    check_error(result, "Q64 zero-copy encoding");
    
    printf("Input: ");
    for (size_t i = 0; i < test_len; i++) {
        printf("%02X ", test_data[i]);
    }
    printf("\n");
    
    printf("Encoded to buffer: ");
    for (size_t i = 0; i < bytes_written; i++) {
        printf("%c", buffer[i]);
    }
    printf(" (%zu bytes written)\n", bytes_written);
    
    assert(bytes_written == test_len * 2);
    printf("✓ Zero-copy encoding successful!\n\n");
}

/**
 * @brief Demonstrate SimHash encoding
 */
static void demo_simhash(void) {
    printf("=== SimHash Encoding ===\n");
    
    // Create a test embedding (simulating float values as bytes)
    const uint8_t embedding[] = {
        100, 200, 50, 150, 75, 125, 225, 25,
        180, 60, 140, 220, 40, 160, 80, 120
    };
    const size_t embedding_len = sizeof(embedding);
    
    printf("Embedding: ");
    for (size_t i = 0; i < embedding_len; i++) {
        printf("%3d ", embedding[i]);
    }
    printf("(%zu values)\n", embedding_len);
    
    // Encode with 64 planes
    const unsigned int planes = 64;
    char* simhash_encoded = NULL;
    
    UubedErrorCode result = uubed_simhash_encode(
        embedding, embedding_len, planes, &simhash_encoded
    );
    check_error(result, "SimHash encoding");
    
    printf("SimHash (64 planes): %s\n", simhash_encoded);
    printf("✓ SimHash encoding successful!\n\n");
    
    uubed_free_string(simhash_encoded);
}

/**
 * @brief Demonstrate Top-K encoding
 */
static void demo_topk(void) {
    printf("=== Top-K Encoding ===\n");
    
    // Create a sparse embedding
    const uint8_t sparse_embedding[] = {
        10, 5, 200, 15, 250, 8, 12, 180, 3, 160,
        7, 140, 240, 20, 190, 6, 220, 25, 170, 9
    };
    const size_t embedding_len = sizeof(sparse_embedding);
    
    printf("Sparse embedding: ");
    for (size_t i = 0; i < embedding_len; i++) {
        printf("%3d ", sparse_embedding[i]);
    }
    printf("\n");
    
    // Encode top-5 values
    const unsigned int k = 5;
    char* topk_encoded = NULL;
    
    UubedErrorCode result = uubed_topk_encode(
        sparse_embedding, embedding_len, k, &topk_encoded
    );
    check_error(result, "Top-K encoding");
    
    printf("Top-%d encoded: %s\n", k, topk_encoded);
    
    // Also try optimized version
    char* topk_optimized = NULL;
    result = uubed_topk_encode_optimized(
        sparse_embedding, embedding_len, k, &topk_optimized
    );
    check_error(result, "Top-K optimized encoding");
    
    printf("Top-%d optimized: %s\n", k, topk_optimized);
    printf("✓ Top-K encoding successful!\n\n");
    
    uubed_free_string(topk_encoded);
    uubed_free_string(topk_optimized);
}

/**
 * @brief Demonstrate Z-order encoding
 */
static void demo_zorder(void) {
    printf("=== Z-order Encoding ===\n");
    
    // Create coordinates-like data
    const uint8_t coordinates[] = {
        100, 150, 200, 120, 180, 160, 140, 190
    };
    const size_t coord_len = sizeof(coordinates);
    
    printf("Coordinates: ");
    for (size_t i = 0; i < coord_len; i++) {
        printf("%3d ", coordinates[i]);
    }
    printf("\n");
    
    char* zorder_encoded = NULL;
    UubedErrorCode result = uubed_zorder_encode(
        coordinates, coord_len, &zorder_encoded
    );
    check_error(result, "Z-order encoding");
    
    printf("Z-order encoded: %s\n", zorder_encoded);
    printf("✓ Z-order encoding successful!\n\n");
    
    uubed_free_string(zorder_encoded);
}

/**
 * @brief Demonstrate error handling
 */
static void demo_error_handling(void) {
    printf("=== Error Handling ===\n");
    
    // Test with invalid input
    char* output = NULL;
    UubedErrorCode result = uubed_q64_encode(NULL, 5, &output);
    
    if (result != UUBED_SUCCESS) {
        const char* error_msg = uubed_get_last_error_message();
        printf("Expected error caught: %s (code: %d)\n", 
               error_msg ? error_msg : "Unknown", result);
    }
    
    // Clear error and test with invalid Q64 string
    uubed_clear_last_error();
    
    uint8_t* decoded = NULL;
    size_t decoded_len = 0;
    result = uubed_q64_decode("invalid!", &decoded, &decoded_len);
    
    if (result != UUBED_SUCCESS) {
        const char* error_msg = uubed_get_last_error_message();
        printf("Expected decode error: %s (code: %d)\n",
               error_msg ? error_msg : "Unknown", result);
    }
    
    printf("✓ Error handling working correctly!\n\n");
}

/**
 * @brief Show library information
 */
static void show_library_info(void) {
    printf("=== Library Information ===\n");
    
    const char* version = uubed_get_version();
    printf("Version: %s\n", version);
    
    int has_simd = uubed_has_simd_support();
    printf("SIMD support: %s\n", has_simd ? "Yes" : "No");
    
    size_t max_embedding = uubed_max_embedding_size();
    printf("Max embedding size: %zu bytes\n", max_embedding);
    
    size_t max_k = uubed_max_k_value();
    printf("Max k value: %zu\n", max_k);
    
    size_t max_planes = uubed_max_simhash_planes();
    printf("Max SimHash planes: %zu\n", max_planes);
    
    printf("\n");
}

/**
 * @brief Main demonstration function
 */
int main(void) {
    printf("uubed C API Demonstration\n");
    printf("=========================\n\n");
    
    show_library_info();
    demo_q64_basic();
    demo_q64_zero_copy();
    demo_simhash();
    demo_topk();
    demo_zorder();
    demo_error_handling();
    
    printf("All demonstrations completed successfully!\n");
    return 0;
}