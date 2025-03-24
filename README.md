# 🚥 Jono

**Jono** is a priority queue based on Redis sorted sets and ULIDs.

+ [Redis sorted sets](https://redis.io/docs/latest/develop/data-types/sorted-sets/) provide:
    + priority ordering through member score sorting
    + lexicographical ordering of equally scored members
+ ULIDs are generated lexicographically ordered by definition so
  they can be used as member values in Redis sorted sets for
  FIFO ordering

These together allow for a simple priority queue where
[`ZPOPMIN`](https://redis.io/docs/latest/commands/zpopmin/) can be used to
atomically get the next job to be processed.
