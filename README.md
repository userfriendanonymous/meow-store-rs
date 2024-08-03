# MeowStore


## How to run locally
- Install meowclient cli tool:
```
cargo install --git https://github.com/userfriendanonymous/meow-store-rs cli
```

- Create an empty folder and `cd` into it.
- This will create default config files:
```
meowstore gen-config -p "."
```
- Create the database in "db" folder:
```
meowstore db create -c "./db_create.toml" -p "./db"
```
- Run the database server:
```
meowstore db run -c "./db_run.toml" -p "./db"
```
- (Optional) Open a new terminal and run the crawler:
```
meowstore crawler run -c "./crawler.toml"
```

custom database, binary custom encoding, meilisearch for full text search, binary tree for exact search, compact memory usage, performance wise, statistics, full data download, open source, token based, restricted access, highly configurable, two major pieces of software: database server and crawler, database server doesn't interact with scratch.mit.edu, crawler sends requests to scratch.mit.edu, gains information and sends it to the database server. Merging databases, memory mapped database, most of time taken writting the custom database, cli tool