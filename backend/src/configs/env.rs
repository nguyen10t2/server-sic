use std::env;
use std::sync::LazyLock;

/// Định nghĩa cấu trúc để lưu trữ các biến môi trường.
/// Sử dụng LazyLock để khởi tạo một lần duy nhất khi truy cập lần đầu
pub struct Env {
    pub ip: String,
    pub port: u16,

    pub mqtt_broker: String,
    pub mqtt_port: u16,
}

/// Hàm lấy giá trị của biến môi trường.
/// Cung cấp giá trị mặc định nếu biến không tồn tại.
/// Trả về lỗi nếu biến không tồn tại và không có giá trị mặc định.
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

impl Env {
    pub fn new() -> Self {
        Env {
            ip: get_env("ip", Some("localhost")).unwrap(),
            port: get_env("port", Some("8080")).unwrap().parse::<u16>().unwrap(),

            mqtt_broker: get_env("mqtt_broker", Some("localhost")).unwrap(),
            mqtt_port: get_env("mqtt_port", Some("1883")).unwrap().parse::<u16>().unwrap(),
        }
    }
}

pub static ENV: LazyLock<Env> = LazyLock::new(|| Env::new());
