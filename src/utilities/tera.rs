use std::{collections::HashMap, time::SystemTime};
use tera::{Filter, Value};

pub fn extract_id() -> impl Filter {
    |value: &Value, _args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        let id = value.get("_id");

        match id {
            None => Err(tera::Error::msg("could not find id field".to_string())),
            Some(id) => Ok(tera::Value::String(
                id["$oid"].to_string().replace("\"", ""),
            )),
        }
    }
}

pub fn digest_asset() -> impl tera::Function {
    let key = SystemTime::now();
    let key = key
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("could not generate asset timestamp");
    let key = key.as_secs().to_string();

    move |args: &HashMap<String, tera::Value>| -> tera::Result<tera::Value> {
        match args.get("file") {
            Some(file) => {
                let mut path = "/assets/".to_string();

                let Some(file) = file.as_str() else {
                    return Err("".to_string().into());
                };

                path.push_str(file);
                path.push_str("?v=");
                path.push_str(&key);

                Ok(path.into())
            }
            None => Err("".to_string().into()),
        }
    }
}
