# How to contribute

This repository uses the [github flow](https://guides.github.com/introduction/flow/). In general follow these steps to
add code.

1. Create a branch for your specific feature. Do not attempt to put too many features in a single branch or PR.
2. Add commits in your branch.
3. Keep your branch up to date with `main`. Regularly merge new commits.
4. Once the feature is ready to be reviewed, merge before creating a pull request. Continuous Integration (CI) will check the added code. If tests pass, someone will review the code and merge it.

Code is tested for formatting, clippy errors and unit tests. You can run local checks using.

```shell
cargo clippy
cargo fmt #to format the code
cargo fmt --all -- --check
```

## Limitations

Due to limitations with feature selection in Cargo, we cannot run commands from the workspace root. Instead if you want
to run the full test suite, run `scripts/test.sh test && scripts/test.sh contract build`

