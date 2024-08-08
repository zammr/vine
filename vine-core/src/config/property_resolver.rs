use config::Config;
use regex::Regex;
use crate::core::Error;

pub trait PropertyResolver {
    fn compute_template_value(&self, template: &str) -> Result<String, Error> {
        let regex = Regex::new("\\$\\{([^}]+)}").unwrap();

        let mut value = template.to_string();
        for cap in regex.captures_iter(template) {
            let cap_value = match cap[1].to_string().split_once(":") {
                // TODO: make more informative error
                None => self.get_string(&cap[1]).ok_or(Error::from("failed to compute property"))?,
                Some((prop, default_template)) => {
                    match self.get_string(prop) {
                        Some(value) => value,
                        None => self.compute_template_value(default_template)?
                    }
                }
            };

            value = value.replace(&cap[0], &cap_value);
        }

        Ok(value)
    }

    fn get_string(&self, key: &str) -> Option<String>;
}


impl PropertyResolver for Config {
    fn get_string(&self, key: &str) -> Option<String> {
        self.get_string(key).ok()
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
    }
}



