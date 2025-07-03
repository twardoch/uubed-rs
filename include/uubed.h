/**
 * @file uubed.h
 * @brief C API for uubed-rs encoding library
 * 
 * This header provides a C-compatible interface to the uubed encoding library,
 * enabling usage from C, C++, and other languages that support C FFI.
 * 
 * @section memory_management Memory Management
 * 
 * The API follows RAII principles:
 * - Strings returned by encoding functions must be freed with `uubed_free_string`
 * - Byte arrays returned by decoding functions must be freed with `uubed_free_bytes`
 * - Error messages are managed internally and do not need to be freed
 * 
 * @section error_handling Error Handling
 * 
 * All functions return error codes. Use `uubed_get_last_error_message()` to get
 * human-readable error descriptions.
 * 
 * @section thread_safety Thread Safety
 * 
 * All functions are thread-safe. Each thread maintains its own error state.
 * 
 * @section example Example Usage
 * 
 * @code{.c}
 * #include "uubed.h"
 * 
 * int main() {
 *     const uint8_t data[] = {0x12, 0x34, 0x56};
 *     char* encoded = NULL;
 *     
 *     UubedErrorCode result = uubed_q64_encode(data, 3, &encoded);
 *     if (result == UUBED_SUCCESS) {
 *         printf("Encoded: %s\n", encoded);
 *         
 *         uint8_t* decoded = NULL;
 *         size_t decoded_len = 0;
 *         
 *         result = uubed_q64_decode(encoded, &decoded, &decoded_len);
 *         if (result == UUBED_SUCCESS) {
 *             printf("Decoded %zu bytes\n", decoded_len);
 *             uubed_free_bytes(decoded, decoded_len);
 *         }
 *         
 *         uubed_free_string(encoded);
 *     } else {
 *         const char* error = uubed_get_last_error_message();
 *         if (error) {
 *             fprintf(stderr, "Error: %s\n", error);
 *         }
 *     }
 *     
 *     return 0;
 * }
 * @endcode
 */

#ifndef UUBED_H
#define UUBED_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

/**
 * @brief Error codes for uubed operations
 */
typedef enum {
    /** Operation succeeded */
    UUBED_SUCCESS = 0,
    /** Q64 encoding/decoding error */
    UUBED_Q64_ERROR = 1,
    /** SimHash computation error */
    UUBED_SIMHASH_ERROR = 2,
    /** Top-k selection error */
    UUBED_TOPK_ERROR = 3,
    /** Z-order encoding error */
    UUBED_ZORDER_ERROR = 4,
    /** Input validation error */
    UUBED_VALIDATION_ERROR = 5,
    /** Memory allocation error */
    UUBED_MEMORY_ERROR = 6,
    /** Internal computation error */
    UUBED_COMPUTATION_ERROR = 7,
    /** Invalid parameter passed to function */
    UUBED_INVALID_PARAMETER = 8,
    /** Buffer too small for operation */
    UUBED_BUFFER_TOO_SMALL = 9,
    /** Unknown/unexpected error */
    UUBED_UNKNOWN_ERROR = 10
} UubedErrorCode;

/**
 * @brief Get the last error message for the current thread
 * 
 * @return Pointer to error message string, or NULL if no error.
 *         The caller must NOT free this pointer - it's managed internally.
 * 
 * @note Thread-safe. Each thread maintains its own error state.
 */
const char* uubed_get_last_error_message(void);

/**
 * @brief Clear the last error message for the current thread
 */
void uubed_clear_last_error(void);

/**
 * @brief Encode binary data using Q64 algorithm
 * 
 * Q64 is a position-safe encoding that prevents position-dependent corruption.
 * The output string will be exactly 2x the input length.
 * 
 * @param data Input binary data
 * @param data_len Length of input data in bytes
 * @param output Pointer to store the encoded string (caller must free with uubed_free_string)
 * 
 * @return Error code indicating success or failure
 * 
 * @note The output string must be freed using `uubed_free_string`.
 * 
 * @example
 * @code{.c}
 * const uint8_t data[] = {0x12, 0x34, 0x56};
 * char* encoded = NULL;
 * UubedErrorCode result = uubed_q64_encode(data, 3, &encoded);
 * if (result == UUBED_SUCCESS) {
 *     printf("Encoded: %s\n", encoded);
 *     uubed_free_string(encoded);
 * }
 * @endcode
 */
UubedErrorCode uubed_q64_encode(const uint8_t* data, size_t data_len, char** output);

/**
 * @brief Decode Q64-encoded string back to binary data
 * 
 * @param encoded Q64-encoded string (null-terminated)
 * @param output Pointer to store decoded data (caller must free with uubed_free_bytes)
 * @param output_len Pointer to store length of decoded data
 * 
 * @return Error code indicating success or failure
 * 
 * @note The output bytes must be freed using `uubed_free_bytes`.
 */
UubedErrorCode uubed_q64_decode(const char* encoded, uint8_t** output, size_t* output_len);

/**
 * @brief Zero-copy Q64 encoding into pre-allocated buffer
 * 
 * This function performs Q64 encoding without allocating memory, writing
 * directly into a caller-provided buffer. The buffer must be at least
 * `data_len * 2` bytes in size.
 * 
 * @param data Input binary data
 * @param data_len Length of input data
 * @param output_buffer Pre-allocated output buffer
 * @param buffer_len Size of output buffer (must be at least data_len * 2)
 * @param bytes_written Pointer to store number of bytes written
 * 
 * @return Error code indicating success or failure
 * 
 * @note No memory allocation occurs. The output is written as bytes, not a null-terminated string.
 */
UubedErrorCode uubed_q64_encode_to_buffer(
    const uint8_t* data,
    size_t data_len,
    uint8_t* output_buffer,
    size_t buffer_len,
    size_t* bytes_written
);

/**
 * @brief Encode embedding using SimHash algorithm
 * 
 * SimHash creates a locality-sensitive hash that preserves similarity
 * relationships in the embedding space.
 * 
 * @param embedding Input embedding data
 * @param embedding_len Length of embedding
 * @param planes Number of hash planes (must be > 0, recommended: 64-256)
 * @param output Pointer to store encoded string (caller must free with uubed_free_string)
 * 
 * @return Error code indicating success or failure
 * 
 * @note Higher plane counts provide better precision but larger output.
 */
UubedErrorCode uubed_simhash_encode(
    const uint8_t* embedding,
    size_t embedding_len,
    unsigned int planes,
    char** output
);

/**
 * @brief Encode embedding using Top-K algorithm
 * 
 * Top-K encoding selects the k largest values and their indices,
 * providing a compressed representation of sparse embeddings.
 * 
 * @param embedding Input embedding data
 * @param embedding_len Length of embedding
 * @param k Number of top elements to select (must be > 0)
 * @param output Pointer to store encoded string (caller must free with uubed_free_string)
 * 
 * @return Error code indicating success or failure
 * 
 * @note Smaller k values provide better compression but may lose information.
 */
UubedErrorCode uubed_topk_encode(
    const uint8_t* embedding,
    size_t embedding_len,
    unsigned int k,
    char** output
);

/**
 * @brief Encode embedding using optimized Top-K algorithm
 * 
 * This is an optimized version of the Top-K algorithm that provides
 * better performance for large embeddings and large k values.
 * 
 * @param embedding Input embedding data
 * @param embedding_len Length of embedding
 * @param k Number of top elements to select (must be > 0)
 * @param output Pointer to store encoded string (caller must free with uubed_free_string)
 * 
 * @return Error code indicating success or failure
 * 
 * @note Recommended for k > 16 or embeddings > 1000 elements.
 */
UubedErrorCode uubed_topk_encode_optimized(
    const uint8_t* embedding,
    size_t embedding_len,
    unsigned int k,
    char** output
);

/**
 * @brief Encode embedding using Z-order (Morton) algorithm
 * 
 * Z-order encoding provides spatial locality preservation, useful
 * for embeddings with spatial or hierarchical structure.
 * 
 * @param embedding Input embedding data
 * @param embedding_len Length of embedding
 * @param output Pointer to store encoded string (caller must free with uubed_free_string)
 * 
 * @return Error code indicating success or failure
 * 
 * @note Best suited for embeddings with spatial or coordinate-like structure.
 */
UubedErrorCode uubed_zorder_encode(
    const uint8_t* embedding,
    size_t embedding_len,
    char** output
);

/**
 * @brief Free a string allocated by uubed encoding functions
 * 
 * @param s String to free (must be allocated by uubed functions)
 * 
 * @warning The pointer must have been returned by an uubed encoding function.
 *          After calling this function, the pointer is invalid and must not be used.
 */
void uubed_free_string(char* s);

/**
 * @brief Free bytes allocated by uubed decoding functions
 * 
 * @param bytes Bytes to free
 * @param len Length of the byte array
 * 
 * @warning The pointer must have been returned by an uubed decoding function.
 */
void uubed_free_bytes(uint8_t* bytes, size_t len);

/**
 * @brief Get version information
 * 
 * @return Version string (do not free - statically allocated)
 */
const char* uubed_get_version(void);

/**
 * @brief Check if SIMD optimizations are available
 * 
 * @return 1 if SIMD is available, 0 otherwise
 */
int uubed_has_simd_support(void);

/**
 * @brief Get maximum supported embedding size
 * 
 * @return Maximum embedding size in bytes
 */
size_t uubed_max_embedding_size(void);

/**
 * @brief Get maximum supported k value for top-k operations
 * 
 * @return Maximum k value
 */
size_t uubed_max_k_value(void);

/**
 * @brief Get maximum supported planes for SimHash operations
 * 
 * @return Maximum planes value
 */
size_t uubed_max_simhash_planes(void);

#ifdef __cplusplus
}
#endif

#endif /* UUBED_H */