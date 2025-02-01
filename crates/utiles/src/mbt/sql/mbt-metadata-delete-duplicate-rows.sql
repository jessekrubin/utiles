-- Deletes from the metadata table all rows that have the same name and value.
DELETE
FROM metadata
WHERE rowid NOT IN (SELECT min_rowid
                    FROM (SELECT MIN(rowid) AS min_rowid
                          FROM metadata
                          GROUP BY name
                          HAVING COUNT(*) > 1
                             AND COUNT(DISTINCT value) = 1));
