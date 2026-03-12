#![allow(dead_code)]

use std::env;

/**
 * Hàm lấy giá trị của biến môi trường 
 * Cung cấp giá trị mặc định nếu biến không tồn tại.
 * Trả về lỗi nếu biến không tồn tại và không có giá trị mặc định.
 */
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