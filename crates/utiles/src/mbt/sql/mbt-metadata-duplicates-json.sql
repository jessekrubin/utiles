WITH
  duplicate_vals AS (
    SELECT
      name,
      json_group_array(value) AS vals
    FROM
      (
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
  )
SELECT
  json_group_object(name, json(vals)) AS metadata
FROM
  duplicate_vals;
