#[cfg(test)]
mod session_tests {
    use crate::config::manager::{Config, ConfigManager};
    use crate::models::session::{AuthType, Session};
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_session_management() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("devlg.toml");

        // Create a new config
        let mut manager = ConfigManager::new(Some(config_path.clone()));
        manager.load().unwrap();
        assert!(manager.config.sessions.is_empty());

        // Add a session
        let session = Session::new(
            "test".to_string(),
            "example.com".to_string(),
            "user".to_string(),
            22,
            AuthType::Key,
            Some(PathBuf::from("~/.ssh/id_rsa")),
            None,
            Some(HashSet::from(["production".to_string(), "web".to_string()])),
        );
        manager.config.add_session(session.clone()).unwrap();
        assert_eq!(manager.config.sessions.len(), 1);

        // Save config to file
        manager.save().unwrap();

        // Load config from file
        let loaded_config: Config =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(loaded_config.sessions.len(), 1);

        // Get the session
        let found_session = manager.config.get_session("test").unwrap();
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
            Some(HashSet::from(["staging".to_string()])),
        );
        manager.config.update_session(modified_session).unwrap();

        // Save modified config
        manager.save().unwrap();

        // Load modified config
        let loaded_modified_config: Config =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert_eq!(loaded_modified_config.sessions.len(), 1);

        // Verify modification
        let updated_session = manager.config.get_session("test").unwrap();
        assert_eq!(updated_session.host, "new.example.com");
        assert_eq!(updated_session.user, "newuser");
        assert_eq!(updated_session.port, 2222);

        // Delete the session
        manager.config.remove_session("test").unwrap();
        assert!(manager.config.sessions.is_empty());

        // Save empty config
        manager.save().unwrap();

        // Load empty config
        let loaded_empty_config: Config =
            toml::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
        assert!(loaded_empty_config.sessions.is_empty());
    }
}
