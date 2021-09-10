# Privi Ink contracts Repository

[![tests](https://github.com/Privi-Protocol/Privi-Ink-Contract/actions/workflows/format-lint-test.yml/badge.svg)](https://github.com/Privi-Protocol/Privi-Ink-Contract/actions/workflows/format-lint-test.yml)
[![build](https://github.com/Privi-Protocol/Privi-Ink-Contract/actions/workflows/pre-release.yml/badge.svg)](https://github.com/Privi-Protocol/Privi-Ink-Contract/actions/workflows/pre-release.yml)

## Testing 

We mainly use [redspot](https://github.com/patractlabs/redspot) for testing, since cross contract calls are not yet 
supported using the cargo-contract testing harness. A useful debugging tool is the redspot explorer, which can be used 
to inspect events:

```
npx redspot explorer
```
