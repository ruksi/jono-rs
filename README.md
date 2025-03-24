# 🚥 Jono

**Jono** is a priority job queue based on Redis sorted sets and ULIDs.

+ Redis sorted sets are used because:
    + sorted sets allow priority ordering through member score
    + equally scored members are lexicographically ordered
+ ULIDs are generated lexicographically ordered by definition so
  they can be used as member values in Redis sorted sets for
  FIFO ordering
