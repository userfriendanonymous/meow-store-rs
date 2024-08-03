# MeowStore

## About
MeowStore is a database storing data from scratch.mit.edu. 
This project is divided into two major software: **Database server** and **crawler** (scraper).
- **Database server**: only stores the data, doesn't interact with the Scratch API. Uses a [custom file memory mapped database engine I made](https://github.com/userfriendanonymous/bindb-rs).
For exact searches (such as getting a user by their username) binary tree structure is used.
Uses [meilisearch](https://www.meilisearch.com/) for full-text search (Such as searching comments or forum posts by their contents).

- **Crawler**: Constantly sends requests to the Scratch API to collect information (about users, projects, forums, etc.). Sends the collected data to the database server.

## Example requests
`{FORMAT}` - `json` or `bin` (binary). Tells whether a request/response should be in JSON or binary format.
Binary en/decoding uses [bincode-rs](https://github.com/bincode-org/bincode).
### Get a user by username
```
GET /users/get_by_name/{USERNAME}/{FORMAT}
```
Example:
```
GET /users/get_by_name/griffpatch/json
```
Response:
```json
{
  "Ok": {
    "name": "griffpatch",
    "id": 1882674,
    "scratch_team": false,

    "loves": 3211809,
    "favorites": 2887670,
    "views": 295423368,
    "remixes": 0,

    "status": "YouTube Tutorials ▶️ www.youtube.com/griffpatch\nI have only 2 other accounts:\n@griffpatch_tutor | @Griffpatch-Academy\nNo 4f4 or f4f sorry\nPlease don't spam: Max 1 ad per person a day",

    "bio": "Got hooked on coding when I was a kid, now I'm a parent and nothing's changed! My day job involves java coding. In my spare time I love making games, being creative & drumming in church."
  }
}
```
### Search users by their bio/status
```
GET /users/search/{QUERY}/{FORMAT}
```
Example:
```
GET /users/search/someone/json
```
Response:
```json
{
  "Ok": [
    {
      "name": "Digital_i5",
      "id": 140519353,
      "scratch_team": false,
      "status": "Hm, topology, PHP, planche push ups, and games.\n\nNot responding to 85% of comments.\nSorry, but absolutely NO F4F. But if someone tells you ...",
      "bio": "Hi, I’m @Digital_i5\nI’m ...",
      "loves": 0,
      "favorites": 0,
      "views": 0,
      "remixes": 0
    },
    {
      "name": "Tsukise-Yune",
      "id": 123973642,
      "scratch_team": false,
      "status": "Is there any meaning in my being alive? lol\nIt probably doesn't make any sense lol\n\n＊someone to cherish＊\n　...",
      "bio": "...",
      "loves": 44,
      "favorites": 38,
      "views": 240,
      "remixes": 0
    }
  ]
}
```

### Insert a user
```
POST /write/{BODY_FORMAT}/{RESPONSE_FORMAT}
```
Example
```
POST /write/json/json
```
Body:
```json
{
    "name": "griffpatch",
    "id": 1882674,
    "scratch_team": false,
    "loves": 3211809,
    "favorites": 2887670,
    "views": 295423368,
    "remixes": 0,
    "status": "Status ...",
    "bio": "Bio ..."
  }
```
Response:

```json
{
    "Ok": false
}
```
(`false` - successfully inserted, `true` - the user was already inserted before.)

## How to run locally
I'm currently not hosting a database server so the only way to try this project is to run it locally.

- Install meowstore cli tool:
```
cargo install --git https://github.com/userfriendanonymous/meow-store-rs meowstore
```

- Create an empty folder and `cd` into it.
- This will create default config files:
```
meowstore gen-config -p "."
```
You can view/edit the created files.

- Create a database in "db" folder:
```
meowstore db create -c "./db_create.toml" -p "./db"
```
- Run the database server:
```
meowstore db run -c "./db_run.toml" -p "./db"
```
- (Optional) Open a new terminal and run a crawler:
```
meowstore crawler run -c "./crawler.toml"
```

Now you can make requests to a running database server.

## Plans
- Add more fields to the users info (history, ...).
- Add endpoints for projects, studios, forums, ...
- Make it possible for the database server to optionally require authentication for `write` / `remove` endpoints.

## Note
I'm currently **not**:
- Crawling/scraping/making requests to scratch.mit.edu.
- Hosting a database server.

## Contributing
You can make contributions, but:
- Some code may be hard to understand, because I didn't yet add documentation comments. (I'm working on this).

This project also has these dependencies that I'm working on:
- [bindb-rs](https://github.com/userfriendanonymous/bindb-rs): Custom database engine used by this project.
- [rs2s](https://github.com/userfriendanonymous/rs2s-rs): Scratch API client.
- [http-input](https://github.com/userfriendanonymous/http-input-rs): A small library that allows describing an HTTP request without actually sending it.