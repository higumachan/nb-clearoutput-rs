# fast-notebook-clear-output

A reasonably fast jupyter notebook clear output tool.

## Why reasonably fast?

Because it is implemented in Rust, it is faster than jupyter nbconvert. However, it is not blazing because I have not tried to make it faster on Rust.

```
❯ hyperfine "./target/release/nb-clearoutput-rs  stdout assets/notebook/benchmark.ipynb" "pipx run jupyter nbconvert --ClearOutputPreprocessor.enabled=True  --to notebook  --stdout assets/notebook/benchmark.ipynb"
Benchmark 1: ./target/release/nb-clearoutput-rs  stdout assets/notebook/benchmark.ipynb
  Time (mean ± σ):     509.6 ms ±  10.4 ms    [User: 350.8 ms, System: 151.5 ms]
  Range (min … max):   494.9 ms … 525.4 ms    10 runs

Benchmark 2: pipx run jupyter nbconvert --ClearOutputPreprocessor.enabled=True  --to notebook  --stdout assets/notebook/benchmark.ipynb
  Time (mean ± σ):      2.070 s ±  0.231 s    [User: 1.573 s, System: 0.392 s]
  Range (min … max):    1.980 s …  2.726 s    10 runs
```

## Install

### Cargo Install

```bash
cargo install fast-notebook-clear-output
```

## Usage

Output stdout

```bash
nbclo stdout {notebook.ipynb}
```

Replace the notebook

```bash
nbclo inplace {notebook.ipynb}
```
