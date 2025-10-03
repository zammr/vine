use config::Config;
use regex::Regex;
use crate::core::Error;

pub trait PropertyResolver {
    fn compute_template_value(&self, template: &str) -> Result<String, Error> {
        let regex = Regex::new("\\$\\{([^}]+)}").unwrap();

        let mut value = template.to_string();
        for cap in regex.captures_iter(template) {
            let cap_value = match cap[1].to_string().split_once(":") {
                None => {
                    // ${property} - no default, must exist
                    self.get_string(&cap[1])
                        .ok_or(Error::from(format!("Property '{}' not found and no default value provided", &cap[1])))?
                },
                Some((prop, default_template)) => {
                    // ${property:default} - use default if property doesn't exist
                    match self.get_string(prop) {
                        Some(value) => value,
                        None => {
                            // Support empty defaults: ${property:}
                            if default_template.is_empty() {
                                String::new()
                            } else {
                                // Recursively resolve default (supports ${prop:${nested}})
                                self.compute_template_value(default_template)?
                            }
                        }
                    }
                }
            };

            value = value.replace(&cap[0], &cap_value);
        }

        Ok(value)
    }

    fn compute_template_value_as_bool(&self, template: &str) -> Result<bool, Error> {
        self.compute_template_value(template)?.parse::<bool>()
            .map_err(|e| Error::from(format!("failed to parse bool: {}", e)))
    }

    fn compute_template_value_as_i8(&self, template: &str) -> Result<i8, Error> {
        self.compute_template_value(template)?.parse::<i8>()
            .map_err(|e| Error::from(format!("failed to parse i8: {}", e)))
    }

    fn compute_template_value_as_i16(&self, template: &str) -> Result<i16, Error> {
        self.compute_template_value(template)?.parse::<i16>()
            .map_err(|e| Error::from(format!("failed to parse i16: {}", e)))
    }

    fn compute_template_value_as_i32(&self, template: &str) -> Result<i32, Error> {
        self.compute_template_value(template)?.parse::<i32>()
            .map_err(|e| Error::from(format!("failed to parse i32: {}", e)))
    }

    fn compute_template_value_as_i64(&self, template: &str) -> Result<i64, Error> {
        self.compute_template_value(template)?.parse::<i64>()
            .map_err(|e| Error::from(format!("failed to parse i64: {}", e)))
    }

    fn compute_template_value_as_u8(&self, template: &str) -> Result<u8, Error> {
        self.compute_template_value(template)?.parse::<u8>()
            .map_err(|e| Error::from(format!("failed to parse u8: {}", e)))
    }

    fn compute_template_value_as_u16(&self, template: &str) -> Result<u16, Error> {
        self.compute_template_value(template)?.parse::<u16>()
            .map_err(|e| Error::from(format!("failed to parse u16: {}", e)))
    }

    fn compute_template_value_as_u32(&self, template: &str) -> Result<u32, Error> {
        self.compute_template_value(template)?.parse::<u32>()
            .map_err(|e| Error::from(format!("failed to parse u32: {}", e)))
    }

    fn compute_template_value_as_u64(&self, template: &str) -> Result<u64, Error> {
        self.compute_template_value(template)?.parse::<u64>()
            .map_err(|e| Error::from(format!("failed to parse u64: {}", e)))
    }

    fn compute_template_value_as_f32(&self, template: &str) -> Result<f32, Error> {
        self.compute_template_value(template)?.parse::<f32>()
            .map_err(|e| Error::from(format!("failed to parse f32: {}", e)))
    }

    fn compute_template_value_as_f64(&self, template: &str) -> Result<f64, Error> {
        self.compute_template_value(template)?.parse::<f64>()
            .map_err(|e| Error::from(format!("failed to parse f64: {}", e)))
    }

    fn get_string(&self, key: &str) -> Option<String>;

    fn get_bool(&self, key: &str) -> Option<bool>;

    fn get_i8(&self, key: &str) -> Option<i8>;

    fn get_i16(&self, key: &str) -> Option<i16>;

    fn get_i32(&self, key: &str) -> Option<i32>;

    fn get_i64(&self, key: &str) -> Option<i64>;

    fn get_u8(&self, key: &str) -> Option<u8>;

    fn get_u16(&self, key: &str) -> Option<u16>;

    fn get_u32(&self, key: &str) -> Option<u32>;

    fn get_u64(&self, key: &str) -> Option<u64>;

    fn get_f32(&self, key: &str) -> Option<f32>;

    fn get_f64(&self, key: &str) -> Option<f64>;
}


impl PropertyResolver for Config {
    fn get_string(&self, key: &str) -> Option<String> {
        self.get_string(key).ok()
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_bool(key).ok()
    }

    fn get_i8(&self, key: &str) -> Option<i8> {
        self.get_int(key).ok().and_then(|v| i8::try_from(v).ok())
    }

    fn get_i16(&self, key: &str) -> Option<i16> {
        self.get_int(key).ok().and_then(|v| i16::try_from(v).ok())
    }

    fn get_i32(&self, key: &str) -> Option<i32> {
        self.get_int(key).ok().and_then(|v| i32::try_from(v).ok())
    }

    fn get_i64(&self, key: &str) -> Option<i64> {
        self.get_int(key).ok()
    }

    fn get_u8(&self, key: &str) -> Option<u8> {
        self.get_int(key).ok().and_then(|v| u8::try_from(v).ok())
    }

    fn get_u16(&self, key: &str) -> Option<u16> {
        self.get_int(key).ok().and_then(|v| u16::try_from(v).ok())
    }

    fn get_u32(&self, key: &str) -> Option<u32> {
        self.get_int(key).ok().and_then(|v| u32::try_from(v).ok())
    }

    fn get_u64(&self, key: &str) -> Option<u64> {
        self.get_int(key).ok().and_then(|v| u64::try_from(v).ok())
    }

    fn get_f32(&self, key: &str) -> Option<f32> {
        self.get_float(key).ok().map(|v| v as f32)
    }

    fn get_f64(&self, key: &str) -> Option<f64> {
        self.get_float(key).ok()
    }
}

#[cfg(test)]
mod tests {
    use config::Config;
    use crate::config::PropertyResolver;

    #[test]
    fn should_template_correct_values() {
        let config = Config::builder()
            .set_default("prop1", "value1").unwrap()
            .set_default("prop2", "value2").unwrap()
            .build().unwrap();

        let value = config.compute_template_value("value").unwrap();
        assert_eq!(value.as_str(), "value");

        let value = config.compute_template_value("${prop1}").unwrap();
        assert_eq!(value.as_str(), "value1");

        let value = config.compute_template_value("${prop1}_${prop2}").unwrap();
        assert_eq!(value.as_str(), "value1_value2");

        let value = config.compute_template_value("${prop1:default_value1}_${prop3:default_value3}").unwrap();
        assert_eq!(value.as_str(), "value1_default_value3");

        // Test empty default value
        let value = config.compute_template_value("${missing:}").unwrap();
        assert_eq!(value.as_str(), "");

        // Test missing property without default should fail
        let result = config.compute_template_value("${missing_prop}");
        assert!(result.is_err());
    }

    #[test]
    fn should_compute_template_value_as_bool() {
        let config = Config::builder()
            .set_default("bool_true", "true").unwrap()
            .set_default("bool_false", "false").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_bool("true").unwrap(), true);
        assert_eq!(config.compute_template_value_as_bool("false").unwrap(), false);
        assert_eq!(config.compute_template_value_as_bool("${bool_true}").unwrap(), true);
        assert_eq!(config.compute_template_value_as_bool("${bool_false}").unwrap(), false);
        assert_eq!(config.compute_template_value_as_bool("${missing:true}").unwrap(), true);
        assert!(config.compute_template_value_as_bool("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_i8() {
        let config = Config::builder()
            .set_default("num", "42").unwrap()
            .set_default("negative", "-10").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_i8("42").unwrap(), 42);
        assert_eq!(config.compute_template_value_as_i8("-10").unwrap(), -10);
        assert_eq!(config.compute_template_value_as_i8("${num}").unwrap(), 42);
        assert_eq!(config.compute_template_value_as_i8("${negative}").unwrap(), -10);
        assert_eq!(config.compute_template_value_as_i8("${missing:127}").unwrap(), 127);
        assert!(config.compute_template_value_as_i8("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_i16() {
        let config = Config::builder()
            .set_default("num", "1000").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_i16("1000").unwrap(), 1000);
        assert_eq!(config.compute_template_value_as_i16("${num}").unwrap(), 1000);
        assert_eq!(config.compute_template_value_as_i16("${missing:32767}").unwrap(), 32767);
        assert!(config.compute_template_value_as_i16("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_i32() {
        let config = Config::builder()
            .set_default("num", "100000").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_i32("100000").unwrap(), 100000);
        assert_eq!(config.compute_template_value_as_i32("${num}").unwrap(), 100000);
        assert_eq!(config.compute_template_value_as_i32("${missing:-100000}").unwrap(), -100000);
        assert!(config.compute_template_value_as_i32("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_i64() {
        let config = Config::builder()
            .set_default("num", "9000000000").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_i64("9000000000").unwrap(), 9_000_000_000);
        assert_eq!(config.compute_template_value_as_i64("${num}").unwrap(), 9_000_000_000);
        assert_eq!(config.compute_template_value_as_i64("${missing:-9000000000}").unwrap(), -9_000_000_000);
        assert!(config.compute_template_value_as_i64("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_u8() {
        let config = Config::builder()
            .set_default("num", "42").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_u8("42").unwrap(), 42);
        assert_eq!(config.compute_template_value_as_u8("${num}").unwrap(), 42);
        assert_eq!(config.compute_template_value_as_u8("${missing:255}").unwrap(), 255);
        assert!(config.compute_template_value_as_u8("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_u16() {
        let config = Config::builder()
            .set_default("num", "1000").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_u16("1000").unwrap(), 1000);
        assert_eq!(config.compute_template_value_as_u16("${num}").unwrap(), 1000);
        assert_eq!(config.compute_template_value_as_u16("${missing:65535}").unwrap(), 65535);
        assert!(config.compute_template_value_as_u16("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_u32() {
        let config = Config::builder()
            .set_default("num", "100000").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_u32("100000").unwrap(), 100000);
        assert_eq!(config.compute_template_value_as_u32("${num}").unwrap(), 100000);
        assert_eq!(config.compute_template_value_as_u32("${missing:200000}").unwrap(), 200000);
        assert!(config.compute_template_value_as_u32("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_u64() {
        let config = Config::builder()
            .set_default("num", "9000000000").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_u64("9000000000").unwrap(), 9_000_000_000);
        assert_eq!(config.compute_template_value_as_u64("${num}").unwrap(), 9_000_000_000);
        assert_eq!(config.compute_template_value_as_u64("${missing:18000000000}").unwrap(), 18_000_000_000);
        assert!(config.compute_template_value_as_u64("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_f32() {
        let config = Config::builder()
            .set_default("num", "3.14").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_f32("3.14").unwrap(), 3.14_f32);
        assert_eq!(config.compute_template_value_as_f32("${num}").unwrap(), 3.14_f32);
        assert_eq!(config.compute_template_value_as_f32("${missing:-2.5}").unwrap(), -2.5_f32);
        assert!(config.compute_template_value_as_f32("invalid").is_err());
    }

    #[test]
    fn should_compute_template_value_as_f64() {
        let config = Config::builder()
            .set_default("num", "3.141592653589793").unwrap()
            .build().unwrap();

        assert_eq!(config.compute_template_value_as_f64("3.141592653589793").unwrap(), 3.141592653589793);
        assert_eq!(config.compute_template_value_as_f64("${num}").unwrap(), 3.141592653589793);
        assert_eq!(config.compute_template_value_as_f64("${missing:-2.718281828459045}").unwrap(), -2.718281828459045);
        assert!(config.compute_template_value_as_f64("invalid").is_err());
    }

    #[test]
    fn should_get_string() {
        let config = Config::builder()
            .set_default("key", "value").unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_string(&config, "key"), Some("value".to_string()));
        assert_eq!(PropertyResolver::get_string(&config, "missing"), None);
    }

    #[test]
    fn should_get_bool() {
        let config = Config::builder()
            .set_default("bool_true", true).unwrap()
            .set_default("bool_false", false).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_bool(&config, "bool_true"), Some(true));
        assert_eq!(PropertyResolver::get_bool(&config, "bool_false"), Some(false));
        assert_eq!(PropertyResolver::get_bool(&config, "missing"), None);
    }

    #[test]
    fn should_get_i8() {
        let config = Config::builder()
            .set_default("num", 42).unwrap()
            .set_default("max", 127).unwrap()
            .set_default("min", -128).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_i8(&config, "num"), Some(42));
        assert_eq!(PropertyResolver::get_i8(&config, "max"), Some(127));
        assert_eq!(PropertyResolver::get_i8(&config, "min"), Some(-128));
        assert_eq!(PropertyResolver::get_i8(&config, "missing"), None);
    }

    #[test]
    fn should_get_i16() {
        let config = Config::builder()
            .set_default("num", 1000).unwrap()
            .set_default("max", 32767).unwrap()
            .set_default("min", -32768).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_i16(&config, "num"), Some(1000));
        assert_eq!(PropertyResolver::get_i16(&config, "max"), Some(32767));
        assert_eq!(PropertyResolver::get_i16(&config, "min"), Some(-32768));
        assert_eq!(PropertyResolver::get_i16(&config, "missing"), None);
    }

    #[test]
    fn should_get_i32() {
        let config = Config::builder()
            .set_default("num", 100000).unwrap()
            .set_default("negative", -100000).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_i32(&config, "num"), Some(100000));
        assert_eq!(PropertyResolver::get_i32(&config, "negative"), Some(-100000));
        assert_eq!(PropertyResolver::get_i32(&config, "missing"), None);
    }

    #[test]
    fn should_get_i64() {
        let config = Config::builder()
            .set_default("num", 9_000_000_000_i64).unwrap()
            .set_default("negative", -9_000_000_000_i64).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_i64(&config, "num"), Some(9_000_000_000));
        assert_eq!(PropertyResolver::get_i64(&config, "negative"), Some(-9_000_000_000));
        assert_eq!(PropertyResolver::get_i64(&config, "missing"), None);
    }

    #[test]
    fn should_get_u8() {
        let config = Config::builder()
            .set_default("num", 42).unwrap()
            .set_default("max", 255).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_u8(&config, "num"), Some(42));
        assert_eq!(PropertyResolver::get_u8(&config, "max"), Some(255));
        assert_eq!(PropertyResolver::get_u8(&config, "missing"), None);
    }

    #[test]
    fn should_get_u16() {
        let config = Config::builder()
            .set_default("num", 1000).unwrap()
            .set_default("max", 65535).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_u16(&config, "num"), Some(1000));
        assert_eq!(PropertyResolver::get_u16(&config, "max"), Some(65535));
        assert_eq!(PropertyResolver::get_u16(&config, "missing"), None);
    }

    #[test]
    fn should_get_u32() {
        let config = Config::builder()
            .set_default("num", 100000).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_u32(&config, "num"), Some(100000));
        assert_eq!(PropertyResolver::get_u32(&config, "missing"), None);
    }

    #[test]
    fn should_get_u64() {
        let config = Config::builder()
            .set_default("num", 9_000_000_000_u64).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_u64(&config, "num"), Some(9_000_000_000));
        assert_eq!(PropertyResolver::get_u64(&config, "missing"), None);
    }

    #[test]
    fn should_get_f32() {
        let config = Config::builder()
            .set_default("num", 3.14).unwrap()
            .set_default("negative", -2.5).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_f32(&config, "num"), Some(3.14_f32));
        assert_eq!(PropertyResolver::get_f32(&config, "negative"), Some(-2.5_f32));
        assert_eq!(PropertyResolver::get_f32(&config, "missing"), None);
    }

    #[test]
    fn should_get_f64() {
        let config = Config::builder()
            .set_default("num", 3.141592653589793).unwrap()
            .set_default("negative", -2.718281828459045).unwrap()
            .build().unwrap();

        assert_eq!(PropertyResolver::get_f64(&config, "num"), Some(3.141592653589793));
        assert_eq!(PropertyResolver::get_f64(&config, "negative"), Some(-2.718281828459045));
        assert_eq!(PropertyResolver::get_f64(&config, "missing"), None);
    }
}



