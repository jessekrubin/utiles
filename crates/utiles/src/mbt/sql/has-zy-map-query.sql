SELECT (SELECT COUNT(*) = 1
        FROM sqlite_schema
        WHERE name = '_zy_map'
          AND type = 'table') AND (SELECT COUNT(*) = 3
                                   FROM PRAGMA_TABLE_INFO('_zy_map')
                                   WHERE ((name = 'z' AND type = 'INTEGER')
                                       OR (name = 'y' AND type = 'INTEGER')
                                       OR (name = 'yup' AND type = 'INTEGER')
                                             ))
           AS has_zy_map;
