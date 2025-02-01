-- Returns metadata rows that have duplicate names, but different values
-- as a JSON object that looks like this:
-- {
--     "name1": ["value1", "value2", ...],
--     "name2": ["value1", "value2", ...],
-- }
WITH
    duplicate_vals AS (
        SELECT
            name,
            json_group_array(value) AS vals
        FROM (
                 SELECT
                     name,
                     value
                 FROM
                     metadata
                 WHERE
                     name IN (
                         SELECT
                             name
                         FROM
                             metadata
                         GROUP BY
                             name
                         HAVING
                             count(*) > 1
                     )
                 ORDER BY
                     name
             )
        GROUP BY
            name
        ORDER BY
            name
    ),
    aggregated AS (
        SELECT
            json_group_object(name, json(vals)) AS metadata,
            COUNT(*) AS rowcount   -- how many rows did we actually aggregate?
        FROM
            duplicate_vals
    )
SELECT
    metadata
FROM
    aggregated
WHERE
    rowcount > 0;
