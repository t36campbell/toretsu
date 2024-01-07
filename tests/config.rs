mod tests {
    use confique::json5::{template as json5_template, FormatOptions as Json5FormatOptions};
    use confique::toml::{template as toml_template, FormatOptions as TomlFormatOptions};
    use confique::yaml::{template as yaml_template, FormatOptions as YamlFormatOptions};
    use toretsu::config::Config;

    // #[test]
    #[allow(dead_code)]
    fn generate_conf() {
        let config = Config::new();

        assert_eq!(config.redis_host, "localhost");
        assert_eq!(config.redis_port, 6379);

        let json5 = json5_template::<Config>(Json5FormatOptions::default());
        std::fs::write("example.json", json5).expect("Unable to write file");

        let toml = toml_template::<Config>(TomlFormatOptions::default());
        std::fs::write("example.toml", toml).expect("Unable to write file");

        let yaml = yaml_template::<Config>(YamlFormatOptions::default());
        std::fs::write("example.yaml", yaml).expect("Unable to write file");
    }

    #[test]
    fn new_conf() {
        let config = Config::new();

        assert_eq!(config.redis_host, "localhost");
        assert_eq!(config.redis_port, 6379);
    }
}
