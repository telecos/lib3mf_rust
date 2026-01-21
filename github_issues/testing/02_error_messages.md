---
name: Improve Error Messages with Codes and Context
about: Enhance error messages for better debugging
title: 'Improve Error Messages with Error Codes and File Context'
labels: 'quality, priority:medium, usability'
assignees: ''
---

## Description

Current error messages are functional but could be more helpful for debugging. Adding error codes, file context, and actionable suggestions would significantly improve developer experience.

## Current State

- ✅ Error messages exist for validation failures
- ✅ Error enum with different variants
- ❌ No error codes for categorization
- ❌ Limited context (no line numbers or file locations where possible)
- ❌ No suggestions for common errors
- ❌ Error documentation incomplete

## Impact

- Harder to debug parsing/validation failures
- Users may not understand what went wrong
- No easy way to programmatically handle specific errors
- Difficult to search for solutions

## Expected Outcome

1. **Error Codes**:
   ```rust
   #[derive(Debug)]
   pub enum Error {
       InvalidModel {
           code: &'static str,  // E.g., "E0001"
           message: String,
           context: Option<ErrorContext>,
       },
       // ... other variants with codes
   }
   
   pub struct ErrorContext {
       pub file: Option<String>,
       pub line: Option<usize>,
       pub column: Option<usize>,
       pub hint: Option<String>,
   }
   ```

2. **Better Messages**:
   ```
   Before:
   "Object 5 references non-existent color group ID: 99"
   
   After:
   Error E0234: Invalid color group reference
   Object ID 5 references color group 99, which doesn't exist in resources.
   
   Available color groups: [1, 2, 3, 4]
   Hint: Check that all referenced color groups are defined in <m:colorgroup> elements.
   ```

3. **Error Code Documentation**:
   Create `ERROR_CODES.md` documenting each error code with:
   - What it means
   - Common causes
   - How to fix
   - Examples

## Implementation Notes

**Error Code Scheme**:
- `E0xxx` - Parsing errors (XML, ZIP, OPC)
- `E1xxx` - Validation errors (structure, references)
- `E2xxx` - Extension errors (unsupported, invalid)
- `E3xxx` - I/O errors

**Context Extraction**:
- XML parsing: Use `quick-xml` position tracking
- Validation: Track which object/triangle/element
- Include file paths when known

**Display Implementation**:
```rust
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Error::InvalidModel { code, message, context } => {
                write!(f, "Error {}: {}", code, message)?;
                if let Some(ctx) = context {
                    if let Some(hint) = &ctx.hint {
                        write!(f, "\nHint: {}", hint)?;
                    }
                }
                Ok(())
            }
            // ... other variants
        }
    }
}
```

## Acceptance Criteria

- [ ] Error codes added to all error variants
- [ ] Error codes documented in `ERROR_CODES.md`
- [ ] Context (file/line) included where possible
- [ ] Helpful hints added to common errors
- [ ] Error messages reference spec sections
- [ ] Display formatting improved
- [ ] Tests updated for new error format
- [ ] Documentation updated

## Example Error Codes

- `E0001` - File not found or cannot open
- `E0002` - Invalid ZIP archive
- `E0003` - Missing required file (3dmodel.model)
- `E1001` - Duplicate object ID
- `E1002` - Invalid vertex index (out of bounds)
- `E1003` - Degenerate triangle
- `E1004` - Invalid material reference
- `E2001` - Unsupported extension required
- `E2002` - Invalid extension namespace

## Benefits

- Easier debugging for users
- Better error messages in logs
- Programmatic error handling possible
- Searchable error codes
- Professional library feel

## References

- Rust Error Handling Guidelines
- Error codes from other parsers (e.g., rustc, clippy)

## Related Issues

- Negative Test Conformance (better errors help debugging)
- Documentation Improvements

## Priority

**Medium** - Quality of life improvement, helpful for users but not blocking functionality.

## Effort Estimate

**Small-Medium (2-4 days)** - Refactor error types, add context, write documentation.
