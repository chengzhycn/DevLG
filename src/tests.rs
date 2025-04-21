#[cfg(test)]
mod session_tests {
    use crate::config::manager::Config;
    use crate::models::session::{AuthType, Session};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_session_management() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("devlg.toml");

        // Create a new config
        let mut config = Config::new();
        assert!(config.sessions.is_empty());

        // Add a session
        let session = Session::new(
            "test".to_string(),
            "example.com".to_string(),
            "user".to_string(),
            22,
            AuthType::Key,
            Some(PathBuf::from("~/.ssh/id_rsa")),
            None,
            Some(vec!["production".to_string(), "web".to_string()]),
        );
        assert!(config.add_session(session.clone()).is_ok());
        assert_eq!(config.sessions.len(), 1);

        // Save config to file
        std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();

        // Load config from file
        let loaded_config: Config =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(loaded_config.sessions.len(), 1);

        // Get the session
        let found_session = config.get_session("test").unwrap();
        assert_eq!(found_session.name, "test");
        assert_eq!(found_session.host, "example.com");
        assert_eq!(found_session.user, "user");
        assert_eq!(found_session.port, 22);

        // Modify the session
        let modified_session = Session::new(
            "test".to_string(),
            "new.example.com".to_string(),
            "newuser".to_string(),
            2222,
            AuthType::Password,
            None,
            Some("password123".to_string()),
            Some(vec!["staging".to_string()]),
        );
        assert!(config.update_session(modified_session).is_ok());

        // Save modified config
        std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();

        // Load modified config
        let loaded_modified_config: Config =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(loaded_modified_config.sessions.len(), 1);

        // Verify modification
        let updated_session = config.get_session("test").unwrap();
        assert_eq!(updated_session.host, "new.example.com");
        assert_eq!(updated_session.user, "newuser");
        assert_eq!(updated_session.port, 2222);

        // Delete the session
        assert!(config.remove_session("test").is_ok());
        assert!(config.sessions.is_empty());

        // Save empty config
        std::fs::write(&config_path, toml::to_string(&config).unwrap()).unwrap();

        // Load empty config
        let loaded_empty_config: Config =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(loaded_empty_config.sessions.is_empty());
    }
}
