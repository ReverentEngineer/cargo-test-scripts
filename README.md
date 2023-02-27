# cargo-test-scripts

Run test scripts based off metadata descriptions in Cargot.toml and writes
the results in JUnit format.

Example entries:

```toml
[[package.metadata.test-script]]
name = "test-something"
timeout = 1000 # Timeout in milliseconds
script = [
	"echo hello"
]
[[package.metadata.test-script]]
name = "test-something-else"
script = [
	"echo goodbye"
]
```
