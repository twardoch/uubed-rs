---
this_file: RELEASE_WORKFLOW.md
---
# Build and release

Run `./build.sh` for workspace checks and a native wheel. Run the enclosing
workspace `../../test.sh` for Python and both consumer integrations.

`./publish.sh` publishes built wheels when deliberately invoked. This consolidation
does not publish packages or create Git tags. All crate versions inherit Cargo.toml.
