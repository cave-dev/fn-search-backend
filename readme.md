# fn-search-backend

## getting started

#### setup the database

Install postgres

Create a user and database in postgres

Copy ./config.toml.example to ./config.toml and fill it out

#### run DB migrations

```bash
cd db
# all arguments after the -- are for our program, not cargo
# to see the help for our program, run:
cargo run -- -h
# we must specify a path to the configuration file
cargo run -- -c ../config.toml
```

## running tests

each subproject has their own tests

```bash
# to run tests for the root project
cargo test

# to run tests for the DB
cd db
cargo test
cd ..

# to run tests for the web
cd web
cargo test
cd ..

# etc...
```
