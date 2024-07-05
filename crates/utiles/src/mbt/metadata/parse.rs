use serde_json::Value;
use tracing::error;

use crate::errors::UtilesResult;
use crate::mbt::{MbtMetadataRow, MbtilesMetadataRowParsed, MbtilesMetadataRows};
use crate::UtilesError;

pub fn parse_metadata_json_value(val: Value) -> UtilesResult<MbtilesMetadataRows> {
    match val {
        Value::Array(arr) => {
            let mut rows = Vec::new();
            for value in arr {
                let row_res = serde_json::from_value::<MbtilesMetadataRowParsed>(value);
                match row_res {
                    Ok(row) => {
                        if let Value::String(value) = row.value {
                            let r = MbtMetadataRow::new(row.name, value);
                            rows.push(r);
                        } else {
                            // if it is not a string then serialize it
                            let value_string = serde_json::to_string(&row.value);
                            match value_string {
                                Ok(value_string) => {
                                    let r = MbtMetadataRow::new(row.name, value_string);
                                    rows.push(r);
                                }
                                Err(e) => {
                                    error!("error serializing value: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("error parsing metadata row: {}", e);
                    }
                }
            }
            Ok(rows)
        }
        Value::Object(map) => {
            let mut rows = Vec::new();
            for (key, value) in map {
                if let Value::String(value) = value {
                    let r = MbtMetadataRow::new(key, value);
                    rows.push(r);
                } else {
                    // if it is not a string then serialize it
                    let value_string = serde_json::to_string(&value);
                    match value_string {
                        Ok(value_string) => {
                            let r = MbtMetadataRow::new(key, value_string);
                            rows.push(r);
                        }
                        Err(e) => {
                            error!("error serializing value: {}", e);
                        }
                    }
                }
            }

            Ok(rows)
        }
        _ => {
            error!("unexpected json value (must be obj/arr): {:?}", val);
            Err(UtilesError::Error(
                "unexpected json value; must be object or array".to_string(),
            ))
        }
    }
}

pub fn parse_metadata_json(json_data: &str) -> UtilesResult<Vec<MbtMetadataRow>> {
    let parsed_res = serde_json::from_str(json_data);
    match parsed_res {
        Ok(data) => parse_metadata_json_value(data),
        Err(e) => {
            error!("error parsing json: {}", e);
            Err(UtilesError::Error("error parsing json".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_parse_json_object() {
        let json_data = r#"
    {
        "maxzoom": 12,
        "minzoom": "9",
        "name": "stuff",
        "tilejson": "3.0.0",
        "tilesize": 256,
        "format": "png"
    }
    "#;
        let rows_parsed = parse_metadata_json(json_data).unwrap();
        // oy vey gotta sort by name bc of the old test(s) I wrote...
        let rows = {
            let mut rows = rows_parsed;
            rows.sort_by(|a, b| a.name.cmp(&b.name));
            rows
        };
        assert_eq!(rows.len(), 6,);

        let expected_json = r#"
        [
          {
            "name": "format",
            "value": "png"
          },
          {
            "name": "maxzoom",
            "value": "12"
          },
          {
            "name": "minzoom",
            "value": "9"
          },
          {
            "name": "name",
            "value": "stuff"
          },
          {
            "name": "tilejson",
            "value": "3.0.0"
          },
          {
            "name": "tilesize",
            "value": "256"
          }
        ]
        "#;
        // have to parse then serialize to compare...
        let expected_rows: Vec<MbtMetadataRow> =
            serde_json::from_str(expected_json).unwrap();
        let expected_rows_json = serde_json::to_string_pretty(&expected_rows).unwrap();
        // stringify the rows
        let rows_value = serde_json::to_value(&rows).unwrap();
        let rows_json = serde_json::to_string_pretty(&rows_value).unwrap();
        assert_eq!(rows_json, expected_rows_json,);
    }

    #[test]
    fn test_parse_metadata_json_array() {
        let json_data = r#"
        [
          {
            "name": "format",
            "value": "png"
          },
          {
            "name": "maxzoom",
            "value": 12
          },
          {
            "name": "minzoom",
            "value": "9"
          },
          {
            "name": "name",
            "value": "stuff"
          },
          {
            "name": "tilejson",
            "value": "3.0.0"
          },
          {
            "name": "tilesize",
            "value": "256"
          }
        ]
        "#;
        let rows = parse_metadata_json(json_data).unwrap();
        let expected_json = r#"
        [
          {
            "name": "format",
            "value": "png"
          },
          {
            "name": "maxzoom",
            "value": "12"
          },
          {
            "name": "minzoom",
            "value": "9"
          },
          {
            "name": "name",
            "value": "stuff"
          },
          {
            "name": "tilejson",
            "value": "3.0.0"
          },
          {
            "name": "tilesize",
            "value": "256"
          }
        ]
        "#;
        // have to parse then serialize to compare...
        let expected_rows: Vec<MbtMetadataRow> =
            serde_json::from_str(expected_json).unwrap();
        let expected_rows_json = serde_json::to_string_pretty(&expected_rows).unwrap();
        // stringify the rows
        let rows_json = serde_json::to_string_pretty(&rows).unwrap();
        assert_eq!(rows_json, expected_rows_json,);
    }
}
