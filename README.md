## Interactive Voting Simulator

A voting simulator based on Ka-Ping Yee's [blog post about election methods](http://zesty.ca/voting/sim/). 

You can try the live version on my blog: [Interactive Voting System Simulator](http://www.furidamu.org/blog/2020/08/23/interactive-voting-system-simulator)

## Development

```sh
./dev.sh
```

## Testing

```sh
cargo test --release
```

## Benchmarking

First, build with 

```sh
cargo bench
```

Note the name of the target (`Running target/release/deps/vote-0f74bdcec4246b04`), and run in under `perf`:

```sh
sudo perf record --call-graph=dwarf \
    target/release/deps/vote-0f74bdcec4246b04 --bench --profile-time 10
```

Then view the report with:

```
sudo perf report --hierarchy -M intel
```

You can navigate with the cursor keys and enter to move into a function, and press `a` to get an annotated disassembly interleaved with source code of the selected function.

For more details, see the [Rust SIMD Performance Guide](https://rust-lang.github.io/packed_simd/perf-guide/prof/linux.html).
