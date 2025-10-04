# Task Completion Checklist

When completing a task, ensure:

1. **Code Quality**
   - [ ] Code follows Rust conventions and project style
   - [ ] No compiler warnings (except explicitly allowed)
   - [ ] Code is properly formatted (`cargo fmt`)
   - [ ] Clippy checks pass (`cargo clippy`)

2. **Testing**
   - [ ] All existing tests pass
   - [ ] New tests added for new functionality (if applicable)
   - [ ] Fixture tests pass for DOM, SSR, and Hydratable modes
   - [ ] Edge cases are covered

3. **Documentation**
   - [ ] Public APIs are documented
   - [ ] README updated if needed
   - [ ] Examples updated if needed

4. **Verification**
   - [ ] Run `cargo test` - all tests pass
   - [ ] Run `cargo clippy` - no warnings
   - [ ] Run `cargo build` - builds successfully
   - [ ] Run examples to verify behavior

5. **Special Considerations for this Project**
   - [ ] Fixture tests match expected output from original babel plugin
   - [ ] Template generation is correct
   - [ ] All three modes (DOM, SSR, Hydratable) work correctly
   - [ ] Event delegation works as expected
