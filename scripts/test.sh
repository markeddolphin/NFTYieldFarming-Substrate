set -e

test() {
  for p in $(find contracts -type f -name "*.toml"); do
    cargo test --manifest-path "$p"
  done
}

if [ $1 = "test" ]; then
  test
fi