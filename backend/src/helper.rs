#![allow(dead_code)]

use std::env;

pub fn get_env(key: &str, default: Option<&str>) -> Result<String, env::VarError> {
    match env::var(key) {
        Ok(val) => Ok(val),
        Err(e) => {
            if let Some(default_val) = default {
                Ok(default_val.into())
            } else {
                Err(e)
            }
        }
    }
    
}