use crate::config::manager::ConfigManager;
use crate::models::session::Template;
use anyhow::Result;

pub async fn handle_template_add(name: String, session: String) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    manager.config.add_template(Template { name, session })?;
    manager.save()?;
    println!("Template added successfully.");
    Ok(())
}

pub async fn handle_template_delete(name: String) -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;
    manager.config.remove_template(&name)?;
    manager.save()?;
    println!("Template deleted successfully.");
    Ok(())
}

pub async fn handle_template_list() -> Result<()> {
    let mut manager = ConfigManager::new(None);
    manager.load()?;

    println!("Available templates:");
    for template in manager.config.templates.iter() {
        println!("{}", template.name);
    }

    Ok(())
}
