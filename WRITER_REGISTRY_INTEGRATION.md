# ExtensionRegistry Writer Integration

## Overview

The `ExtensionRegistry.pre_write_all()` method is now integrated into the 3MF writer flow, allowing extension handlers to prepare or transform data before serialization.

## New API

### `Model::to_writer_with_registry()`

Write a 3MF file with extension registry support. Before serialization, this method calls `pre_write()` on all registered extension handlers.

```rust
use lib3mf::{Model, create_default_registry};
use std::fs::File;

let model = Model::new();
// ... populate model ...

let registry = create_default_registry();
let file = File::create("output.3mf")?;
model.to_writer_with_registry(file, &registry)?;
```

### `Model::write_to_file_with_registry()`

Convenience method for writing to a file path with registry support.

```rust
use lib3mf::{Model, create_default_registry};

let model = Model::new();
// ... populate model ...

let registry = create_default_registry();
model.write_to_file_with_registry("output.3mf", &registry)?;
```

## Backward Compatibility

The existing `to_writer()` and `write_to_file()` methods remain unchanged and continue to work without requiring a registry. This ensures full backward compatibility with existing code.

```rust
// This still works - no pre_write hooks are called
model.to_writer(file)?;
model.write_to_file("output.3mf")?;
```

## Custom Extension Handlers

Extension handlers can implement the `pre_write()` method to prepare data before writing:

```rust
use lib3mf::extension::{ExtensionHandler, ExtensionRegistry};
use lib3mf::{Extension, Model, Result};
use std::sync::Arc;

struct MyExtensionHandler;

impl ExtensionHandler for MyExtensionHandler {
    fn extension_type(&self) -> Extension {
        Extension::Material
    }

    fn validate(&self, _model: &Model) -> Result<()> {
        Ok(())
    }

    fn pre_write(&self, model: &mut Model) -> Result<()> {
        // Prepare or transform data before writing
        println!("Preparing model for writing...");
        
        // Example: Add metadata
        if !model.has_metadata("ProcessedBy") {
            model.metadata.push(lib3mf::model::MetadataEntry::new(
                "ProcessedBy".to_string(),
                "MyExtension".to_string(),
            ));
        }
        
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut model = Model::new();
    // ... populate model ...

    let mut registry = ExtensionRegistry::new();
    registry.register(Arc::new(MyExtensionHandler));

    model.to_writer_with_registry(file, &registry)?;
    Ok(())
}
```

## Use Cases

The `pre_write()` hook can be used for:

1. **Data normalization**: Ensure data is in the correct format before serialization
2. **Metadata injection**: Add processing information or timestamps
3. **Validation**: Perform final validation before writing
4. **Transformation**: Apply any last-minute transformations to the model
5. **Extension-specific preparation**: Prepare extension-specific data structures

## Testing

The feature includes comprehensive integration tests in `tests/writer_registry_integration_test.rs`:

- Test that `pre_write` hooks are called when using registry
- Test backward compatibility (no registry = no hooks)
- Test multiple handlers are all called
- Test file writing with registry
- Test round-trip (write with registry, read back)

Run the tests with:
```bash
cargo test --test writer_registry_integration_test
```

## Example

See `examples/writer_with_registry.rs` for a complete working example:

```bash
cargo run --example writer_with_registry
```

## Design Rationale

The implementation follows the "Option A" approach from the original issue:

- **New methods**: `to_writer_with_registry()` and `write_to_file_with_registry()`
- **Full backward compatibility**: Existing methods work unchanged
- **Follows Rust conventions**: Similar to `from_reader()` vs `from_reader_with_config()`
- **Non-intrusive**: No changes to the `Model` struct itself
- **Flexible**: Users can choose whether to use registry or not

## Related Documentation

- See `src/extension.rs` for `ExtensionHandler` trait and `ExtensionRegistry` documentation
- See `examples/extension_registry_factory.rs` for general registry usage
- See `tests/extension_registry_test.rs` for registry testing patterns
