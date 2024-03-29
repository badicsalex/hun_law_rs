# Hun-Law (Rust version)

A small project for parsing Hungarian Law. A rewrite of the [Python version](https://github.com/badicsalex/hun_law_py) in Rust.

It does the following things:

* Parse PDF files into lines. It does so much more accurately than other pdf2txt implementations.
* Parse "Magyar Közlöny" PDFs into individual Acts
* Separate Acts into structural elements (Articles, subpoints, etc.)
* Parse internal and external references in legal text
* Parse special phrases like amendments and repeals into easy-to-use objects
* Generate plain text, colored text, JSON and YAML version of the parsed documents

## Usage

After cloning the repository, you can run it with `cargo run --release`:

```
cargo run --release -- 2012/100
cargo run --release -- -p act-lines -t plain --mk 2013/31
```

Please see the output of `cargo run --release -- --help` for all options

## Contribution

Feel free to open issues for feature requests or found bugs. Merge Requests are more than welcome too.
