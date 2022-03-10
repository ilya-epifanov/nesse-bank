
# Running

Unit tests + snapshot tests:

    cargo test

10 million records test (takes tens of seconds to complete):

    cargo test --release -- --ignored

Running against a small dataset:

    cargo run --release -- -c memory 10mil-transactions.csv

Running against a large dataset (7x slower on my system but more scalable):

    cargo run --release -- 10mil-transactions.csv
