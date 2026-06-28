#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Error type as a result of parsing an opening hours expression.
 */
typedef enum OHParserError {
  /**
   * The expression parsed correctly
   */
  OH_PARSER_ERROR_OK = 0,
  /**
   * The parser was given a null pointer
   */
  OH_PARSER_ERROR_NULL_PTR = -1,
  /**
   * The given expression contains invalid UTF-8
   */
  OH_PARSER_ERROR_INVALID_UTF8 = -2,
  /**
   * Syntax error
   */
  OH_PARSER_ERROR_SYNTAX = -3,
  /**
   * This error should never be raised. Please report a bug at
   * https://github.com/remi-dupre/opening-hours-rs/issues
   */
  OH_PARSER_ERROR_IMPLEMENTATION = -4,
} OHParserError;

typedef enum OHRuleKind {
  OH_RULE_KIND_OPEN = 0,
  OH_RULE_KIND_CLOSED = 1,
  OH_RULE_KIND_UNKNOWN = 2,
} OHRuleKind;

typedef struct OH OH;

typedef struct OHState {
  enum OHRuleKind kind;
  const char *comment;
} OHState;

/**
 * Parse an opening hours expression and store the result in the given pointer.
 *
 * You will have to free the created object by calling `oh_free`.
 *
 * # Inputs
 *
 * - `expression`: an utf8 string holding an opening hours expression.
 * - `oh_ptr`: a pointer to the pointer the output should be stored at.
 *
 * # Return
 *
 * If the parsing succeeds, this function returns 0 and sets *oh_ptr to a
 * valid pointer to an `OH`.
 *
 * If the parsing fails, this function returns an error < 0.
 *
 * # Safety
 *
 * 1. `expression` must be a valid pointer to a null terminated string.
 *    See https://doc.rust-lang.org/stable/std/ffi/struct.CStr.html#safety
 * 2. `oh_ptr` must be a valid pointer to an OH pointer. You do not have to
 *    allocate data ahead and should not call this on an already parsed OH
 *    object.
 */
enum OHParserError oh_parse(const char *expression, struct OH **oh_ptr);

/**
 * Release memory allocated for this parsed opening hours expression.
 *
 * This will invalidate all pointers created from it, be sure to copy all data
 * that you would use later (for instance comments)
 *
 * # Inputs
 *
 * - `oh`: a valid pointer to a parsed expression.
 *
 * # Safety
 *
 * `oh` must be a valid OH pointer.
 */
void oh_free(struct OH *oh);

/**
 * Get a string representation of this parsed expression.
 *
 * This will output the original expression in a format that is guaranteed to
 * conform to the official specification, but if you want to simplify the
 * expression you may be interested in `oh_normalize`.
 *
 * # Inputs
 *
 * - `oh`: a valid pointer to a parsed expression.
 *
 * # Safety
 *
 * 1. `oh` must be a valid OH pointer.
 * 2. The output pointer is valid as long as oh is not destroyed.
 */
const char *oh_format(struct OH *oh);

/**
 * Normalize the input expression.
 *
 * This will attempt to merge intersecting intervals and will transform the
 * expression in an usually more readable format.
 *
 * # Inputs
 *
 * - `oh`: a valid pointer to a parsed expression.
 *
 * # Safety
 *
 * 1. `oh` must be a valid OH pointer.
 * 2. The original expression will be destroyed, all pointer built from it
 *    may be invalidated.
 */
void oh_normalize(struct OH *oh);

/**
 * Get the current state from the expression.
 *
 * # Inputs
 *
 * - `oh`: a valid pointer to a parsed expression.
 * - `dt`: a positive 64-bits signed local timestamp.
 *
 * # Returns
 *
 * An `OHState` struct which bundles current opening status and a comment
 * string that may be null.
 *
 * If the comment string is not null, it will be valid until `oh` is destroyed.
 *
 * # Safety
 *
 * `oh` must be a valid OH pointer.
 */
struct OHState oh_state_at(struct OH *oh, int64_t dt);

/**
 * Get the current state from the expression.
 *
 * # Inputs
 *
 * - `oh`: a valid pointer to a parsed expression.
 * - `dt`: a positive 64-bits signed local timestamp.
 *
 * # Returns
 *
 * 1. A positive unsigned 64-bits timestamp if a state change will happen.
 * 2. 0 if no state change will happen.
 * 3. -1 if the provided timestamp or pointer is invalid.
 *
 * # Safety
 *
 * `oh` must be a valid OH pointer.
 */
int64_t oh_next_change(const struct OH *oh, int64_t dt);

/**
 * Get a string representation of the state rule kind variants.
 *
 * # Inputs
 *
 * - `rule_kind`: the value that has to be stringified.
 *
 * # Returns
 *
 * A static string representing input rule kind.
 */
const int8_t *oh_rule_kind_format(enum OHRuleKind rule_kind);
