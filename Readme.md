# Local Super Search
Fast searches through big directories through file indexing.

Why should searching for something in a document on my computer take 10 minutes,
while Google runs through terrabytes in milliseconds?
> Indexed searches, not full text

With this goal in mind, the beta version of `Local Super Search` was born, a
Rust based implementation of a "crawler" and indexing Engine (using TextRank) and 
storing the index in a SQLITE database.

## TODO
- for even faster searches, we could reduce the cold-startup time (with connection to database and reading data to mem)
	by dividing into a constantly running local server that talks to a client (that is the ./is (short for indexed search) program) via something like unix sockets
- create database for crawled-commands, so that they can all be replicated in a cron job (something like `./is --re-crawl` )
-[x] only crawl if modification date has changed
- word vectorization to find semantic similarities between keywords and searches
- generate Database path depending on OS (I think mac doesn't have ~/.local/share/) and maybe even Windows

## IDEA
decrease Crawl type by also storing the modification dates of dirs we pass by,
not only files, so that we can eliminate whole branches if no modification has happened.
-> This would however require a bit more storage/complexity and currently crawl time is not
a problem I have really struggled with
