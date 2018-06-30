sqlite3 ':memory:' '
CREATE TABLE tbl (
  a INTEGER, b INTEGER, c INTEGER, d INTEGER,
  PRIMARY KEY (a, b, c)
);

CREATE INDEX tbl_i0 ON tbl (a, c, b);

EXPLAIN QUERY PLAN
SELECT SUM(d)
  FROM tbl
 WHERE a = 100
   AND b BETWEEN 100 AND 200
 GROUP BY c
'
