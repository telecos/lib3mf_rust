# Error Handling in lib3mf

This document describes the error handling approach in lib3mf and how to use the error system effectively.

## Error Codes

All errors in lib3mf include error codes for easy categorization and debugging. Error codes follow the pattern `E<category><number>` where:

- **E1xxx**: I/O and archive errors
- **E2xxx**: XML parsing and structure errors
- **E3xxx**: Model validation errors
- **E4xxx**: Unsupported features

## Error Catalog

### I/O and Archive Errors (E1xxx)

#### E1001: I/O Error
**Common Causes:**
- File not found
- Insufficient permissions
- Disk read error

**Example:**
```
[E1001] I/O error: No such file or directory (os error 2)
```

#### E1002: ZIP Archive Error
**Common Causes:**
- Corrupted ZIP file
- Unsupported compression method
- Truncated archive

**Suggestions:**
- Verify the file is a valid 3MF (ZIP) archive
- Try re-downloading or re-exporting the file

**Example:**
```
[E1002] ZIP error: invalid Zip archive
```

#### E1003: Missing Required File
**Common Causes:**
- Incomplete 3MF package
- Missing 3D model file
- Missing content types file

**Suggestions:**
- Ensure the 3MF archive contains all required files
- Check for [Content_Types].xml and 3D/3dmodel.model files

**Example:**
```
[E1003] Missing required file: 3D/3dmodel.model
```

### XML Errors (E2xxx)

#### E2001: XML Parsing Error
**Common Causes:**
- Malformed XML syntax
- Invalid character encoding
- Unclosed tags

**Example:**
```
[E2001] XML parsing error: unexpected end of file
```

#### E2002: XML Attribute Error
**Common Causes:**
- Missing required attribute
- Invalid attribute value
- Duplicate attribute

**Suggestions:**
- Check the 3MF specification for required attributes
- Verify attribute values are properly formatted

**Example:**
```
[E2002] XML attribute error: Attribute parsing failed: missing attribute 'id'
```

#### E2003: Invalid XML Structure
**Common Causes:**
- Missing required XML elements
- Elements in wrong order
- Invalid element nesting

**Suggestions:**
- Check element hierarchy matches 3MF specification
- Verify required child elements are present

**Example:**
```
[E2003] Invalid XML structure: Element '<vertex>' is missing required attribute 'x'. Check the 3MF specification for required attributes.
```

#### E2004: Invalid 3MF Format
**Common Causes:**
- Non-compliant OPC structure
- Invalid content types
- Missing required OPC relationships

**Suggestions:**
- Verify the file was exported correctly
- Check the 3MF specification for OPC requirements

**Example:**
```
[E2004] Invalid 3MF format: OPC package structure: Missing required file '[Content_Types].xml'
```

### Model Validation Errors (E3xxx)

#### E3001: Invalid Model
**Common Causes:**
- Mesh topology errors (non-manifold, degenerate triangles)
- Invalid object references
- Out-of-bounds vertex indices
- Duplicate object IDs

**Suggestions:**
- Check mesh for manifold edges (each edge shared by at most 2 triangles)
- Verify all vertex indices are within bounds
- Ensure all object IDs are unique and positive
- Validate all references point to existing objects

**Example:**
```
[E3001] Invalid model: Object 1: Triangle 5 is degenerate (v1=0, v2=0, v3=2). All three vertices of a triangle must be distinct. Degenerate triangles with repeated vertices are not allowed in 3MF models.
```

#### E3002: Parse Error
**Common Causes:**
- Invalid number format
- Out-of-range values
- Non-numeric characters in numeric fields

**Suggestions:**
- Verify numeric values use proper format (e.g., "1.5" not "1,5")
- Check for special characters or extra whitespace

**Example:**
```
[E3002] Parse error: Failed to parse 'vertex x coordinate': expected floating-point number, got 'abc'. Verify the value is properly formatted.
```

### Unsupported Features (E4xxx)

#### E4001: Unsupported Feature
**Common Causes:**
- Using 3MF extensions not implemented by this parser
- Future 3MF specification features

**Suggestions:**
- Check if the feature is part of a 3MF extension
- Verify this parser supports the required extensions

**Example:**
```
[E4001] Unsupported feature: custom extension 'http://example.com/custom'
```

#### E4002: Required Extension Not Supported
**Common Causes:**
- 3MF file requires an extension not implemented by this parser
- Extension marked as required in the XML namespace

**Suggestions:**
- Check the file's required extensions in the model XML
- Use a different parser that supports the required extension
- If possible, re-export without the extension

**Example:**
```
[E4002] Required extension not supported: http://schemas.microsoft.com/3dmanufacturing/2015/09/productionextension
```

## Helper Functions

The `Error` type provides several helper functions for creating errors with better context:

### `Error::missing_attribute(element, attribute)`

Creates an error for a missing required XML attribute with helpful suggestions.

**Example:**
```rust
let id = attrs.get("id")
    .ok_or_else(|| Error::missing_attribute("object", "id"))?;
```

**Output:**
```
[E2003] Invalid XML structure: Element '<object>' is missing required attribute 'id'. Check the 3MF specification for required attributes.
```

### `Error::invalid_xml_element(element, message)`

Creates an error with element context.

**Example:**
```rust
return Err(Error::invalid_xml_element("vertex", "Missing required 'x' attribute"));
```

**Output:**
```
[E2003] Invalid XML structure: Element '<vertex>': Missing required 'x' attribute
```

### `Error::invalid_format_context(context, message)`

Creates a format error with contextual information about what part of the format is invalid.

**Example:**
```rust
return Err(Error::invalid_format_context(
    "OPC package structure",
    "Missing required relationship to 3D model"
));
```

**Output:**
```
[E2004] Invalid 3MF format: OPC package structure: Missing required relationship to 3D model
```

### `Error::parse_error_with_context(field_name, value, expected_type)`

Creates a parse error with detailed context about what was being parsed.

**Example:**
```rust
return Err(Error::parse_error_with_context(
    "vertex x coordinate",
    &value_str,
    "floating-point number"
));
```

**Output:**
```
[E3002] Parse error: Failed to parse 'vertex x coordinate': expected floating-point number, got 'abc'. Verify the value is properly formatted.
```

## Best Practices

1. **Always include context**: Use the helper functions to provide meaningful context about where and why an error occurred.

2. **Be specific**: Include object IDs, element names, attribute names, and other identifiers that help locate the problem.

3. **Provide suggestions**: When possible, suggest what the user should check or fix.

4. **Use consistent error codes**: Follow the established error code categories for consistency.

5. **Test error messages**: Ensure error messages are clear and helpful by testing them with actual invalid input.

## Examples

### Good Error Messages ✅

```
[E3001] Invalid model: Object 5: Triangle 12 vertex v1=150 is out of bounds (mesh has 100 vertices, valid indices: 0-99). Vertex indices must reference valid vertices in the mesh. Check that all triangle vertex indices are less than the vertex count.
```

This error message:
- Has an error code (E3001)
- Identifies the object (Object 5)
- Identifies the triangle (Triangle 12)
- Shows the invalid value (v1=150)
- Shows the valid range (0-99)
- Explains the problem
- Provides a suggestion

### Poor Error Messages ❌

```
Invalid vertex index
```

This error message:
- No error code
- No context about which object or triangle
- No information about what the invalid value was
- No suggestion for fixing the issue
