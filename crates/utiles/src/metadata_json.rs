use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

use utiles_core::mbutiles::{MbtilesMetadataRow, MbtilesMetadataRows};

use crate::errors::UtilesResult;
use crate::UtilesError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MbtilesMetadataRowValue {
    /// name TEXT NOT NULL
    pub name: String,
    /// value TEXT NOT NULL
    pub value: Value,
}

pub fn parse_metadata_json_value(val: Value) -> UtilesResult<MbtilesMetadataRows> {
    match val {
        Value::Array(arr) => {
            let mut rows = Vec::new();
            for value in arr {
                let row_res = serde_json::from_value::<MbtilesMetadataRowValue>(value);
                match row_res {
                    Ok(row) => {
                        match row.value {
                            Value::String(value) => {
                                let r = MbtilesMetadataRow::new(row.name, value);
                                rows.push(r);
                            }
                            _ => {
                                // if it is not a string then serialize it
                                let value_string = serde_json::to_string(&row.value);
                                match value_string {
                                    Ok(value_string) => {
                                        let r = MbtilesMetadataRow::new(
                                            row.name,
                                            value_string,
                                        );
                                        rows.push(r);
                                    }
                                    Err(e) => {
                                        error!("error serializing value: {}", e);
                                    }
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
                match value {
                    Value::String(value) => {
                        let r = MbtilesMetadataRow::new(key, value);
                        rows.push(r);
                    }
                    _ => {
                        // if it is not a string then serialize it
                        let value_string = serde_json::to_string(&value);
                        match value_string {
                            Ok(value_string) => {
                                let r = MbtilesMetadataRow::new(key, value_string);
                                rows.push(r);
                            }
                            Err(e) => {
                                error!("error serializing value: {}", e);
                            }
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

pub fn parse_metadata_json(json_data: &str) -> UtilesResult<Vec<MbtilesMetadataRow>> {
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
        let rows = parse_metadata_json(json_data).unwrap();
        assert_eq!(rows.len(), 6,);
        // let expected = [
        //     MbtilesMetadataRow { name: "format", value: "\"png\"" },
        //     MbtilesMetadataRow { name: "maxzoom", value: "12" },
        //     MbtilesMetadataRow { name: "minzoom", value: "\"9\"" },
        //     MbtilesMetadataRow { name: "name", value: "\"stuff\"" },
        //     MbtilesMetadataRow { name: "tilejson", value: "\"3.0.0\"" },
        //     MbtilesMetadataRow { name: "tilesize", value: "256" }
        // ];
        // let expected = vec![
        //     MbtilesMetadataRow::new("format", "png"),
        //     MbtilesMetadataRow::new("maxzoom", "12"),
        //     MbtilesMetadataRow::new("minzoom", "\"9\""),
        //     MbtilesMetadataRow::new("name", "\"stuff\""),
        //     MbtilesMetadataRow::new("tilejson", "\"3.0.0\""),
        //     MbtilesMetadataRow::new("tilesize", "256"),
        // ];
        // assert_eq!(
        //     rows,
        //     expected,
        // );

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
        // let pjson = serde_json::to_string_pretty(&rows).unwrap();
        // println!("{}", pjson);

        println!("{:?}", rows);

        // have to parse then serialize to compare...
        let expected_rows: Vec<MbtilesMetadataRow> =
            serde_json::from_str(expected_json).unwrap();
        let expected_rows_json = serde_json::to_string_pretty(&expected_rows).unwrap();
        // stringify the rows
        let rows_json = serde_json::to_string_pretty(&rows).unwrap();
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
        // assert_eq!(
        //     rows.len(),
        //     6,
        // );
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
        let expected_rows: Vec<MbtilesMetadataRow> =
            serde_json::from_str(expected_json).unwrap();
        let expected_rows_json = serde_json::to_string_pretty(&expected_rows).unwrap();
        // stringify the rows
        let rows_json = serde_json::to_string_pretty(&rows).unwrap();

        // print not escaped if running with `cargo test -- --nocapture`
        // println!("{}", rows_json);
        assert_eq!(rows_json, expected_rows_json,);
    }
}
