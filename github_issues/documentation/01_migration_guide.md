---
name: Create Migration Guide from C++ lib3mf
about: Help users migrate from official C++ library
title: 'Create Migration Guide from C++ lib3mf to Rust Implementation'
labels: 'documentation, priority:low'
assignees: ''
---

## Description

Users familiar with the official C++ lib3mf library may want to migrate to this Rust implementation. A migration guide would help smooth the transition by documenting API differences and providing equivalent code examples.

## Current State

- ✅ Rust library has similar concepts to C++ lib3mf
- ✅ Core functionality parity for reading
- ❌ No migration documentation
- ❌ No API comparison table
- ❌ No side-by-side examples

## Impact

- Harder for C++ lib3mf users to adopt Rust version
- No clear path for migration
- Potential confusion about API differences

## Expected Outcome

Create `MIGRATION_FROM_CPP.md` with:

1. **Overview**:
   - Key differences (Rust vs C++, ownership, error handling)
   - Feature parity matrix
   - When to use which library

2. **API Mapping**:
   ```markdown
   | C++ lib3mf | Rust lib3mf_rust | Notes |
   |------------|------------------|-------|
   | `CModel::New()` | `Model::new()` | Ownership model different |
   | `ReadFromFile()` | `Model::from_reader()` | Uses Read trait |
   | `GetMeshObject()` | `model.resources.objects[i].mesh` | Direct access |
   ```

3. **Side-by-Side Examples**:
   
   **C++ lib3mf**:
   ```cpp
   Lib3MF::PModel model = wrapper->CreateModel();
   Lib3MF::PReader reader = model->QueryReader("3mf");
   reader->ReadFromFile(filename);
   
   for (auto it : model->GetObjects()) {
       auto mesh = it->GetMeshObject();
       // ...
   }
   ```
   
   **Rust lib3mf_rust**:
   ```rust
   let file = File::open(filename)?;
   let model = Model::from_reader(file)?;
   
   for object in &model.resources.objects {
       if let Some(ref mesh) = object.mesh {
           // ...
       }
   }
   ```

4. **Common Migration Patterns**:
   - Error handling (C++ exceptions → Rust Result)
   - Memory management (C++ smart pointers → Rust ownership)
   - Iteration (C++ iterators → Rust for loops)
   - Extension handling
   - Material access

5. **Feature Comparison**:
   - What's supported in both
   - What's only in C++ version
   - What's only in Rust version
   - Planned features

## Implementation Notes

**Sections to Cover**:
- Installation and setup
- Basic reading examples
- Material handling differences
- Extension configuration
- Error handling patterns
- Performance considerations
- Memory usage differences
- Threading/concurrency differences

**Code Examples**:
Provide complete, runnable examples for common tasks:
- Opening and reading a file
- Accessing mesh data
- Working with materials
- Configuring extensions
- Error handling

## Acceptance Criteria

- [ ] `MIGRATION_FROM_CPP.md` created
- [ ] API mapping table complete
- [ ] Side-by-side examples for common operations
- [ ] Error handling migration explained
- [ ] Feature parity matrix included
- [ ] Common migration patterns documented
- [ ] Linked from README.md
- [ ] Examples tested and verified

## Benefits

- Easier adoption for C++ lib3mf users
- Clear expectations about differences
- Reduced learning curve
- Professional documentation

## References

- [Official lib3mf (C++)](https://github.com/3MFConsortium/lib3mf)
- C++ lib3mf API documentation

## Related Issues

- Additional Examples
- Writer Support (feature parity)

## Priority

**Low** - Nice to have but not critical. Most Rust developers will learn the API directly.

## Effort Estimate

**Small (2-3 days)** - Research C++ API, write documentation, create examples.

## Notes

Focus on real-world migration scenarios. Interview users who have used both to understand pain points.
