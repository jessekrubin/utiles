-- Returns the rows that are duplicate (name, value) pairs
SELECT name,
       value,
       COUNT(*) AS count
FROM metadata
GROUP BY name
HAVING COUNT (*)
     > 1
   AND COUNT (DISTINCT value) = 1;
