use std::collections::HashMap;
use std::sync::Mutex;
use keyring::Entry;
use crate::domain::ApplicationError;

const DEFAULT_SERVICE_NAME: &str = "gitverse";

/// Secure credential store wrapping OS Keychain / Keyring with an in-memory fallback for headless or restricted environments.
pub struct CredentialStore {
    service_name: String,
    fallback_store: Mutex<HashMap<String, String>>,
}

impl CredentialStore {
    pub fn new() -> Self {
        Self {
            service_name: DEFAULT_SERVICE_NAME.to_string(),
            fallback_store: Mutex::new(HashMap::new()),
        }
    }

    pub fn with_service_name(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            fallback_store: Mutex::new(HashMap::new()),
        }
    }

    /// Stores a personal access token or OAuth credential securely.
    pub fn store_token(&self, account_identifier: &str, secret_token: &str) -> Result<(), ApplicationError> {
        let entry = Entry::new(&self.service_name, account_identifier);
        match entry {
            Ok(keyring_entry) => {
                match keyring_entry.set_password(secret_token) {
                    Ok(_) => Ok(()),
                    Err(error) => {
                        tracing::warn!(
                            "OS keyring unavailable ({}), utilizing secure fallback storage for account {}",
                            error,
                            account_identifier
                        );
                        let mut fallback = self.fallback_store.lock().map_err(|_| {
                            ApplicationError::PersistenceError("Failed to acquire credential lock".into())
                        })?;
                        fallback.insert(account_identifier.to_string(), secret_token.to_string());
                        Ok(())
                    }
                }
            }
            Err(error) => {
                tracing::warn!(
                    "Failed to initialize OS keyring entry ({}), utilizing secure fallback storage",
                    error
                );
                let mut fallback = self.fallback_store.lock().map_err(|_| {
                    ApplicationError::PersistenceError("Failed to acquire credential lock".into())
                })?;
                fallback.insert(account_identifier.to_string(), secret_token.to_string());
                Ok(())
            }
        }
    }

    /// Retrieves an existing secret token by account identifier.
    pub fn get_token(&self, account_identifier: &str) -> Result<Option<String>, ApplicationError> {
        let entry = Entry::new(&self.service_name, account_identifier);
        match entry {
            Ok(keyring_entry) => {
                match keyring_entry.get_password() {
                    Ok(secret) => Ok(Some(secret)),
                    Err(keyring::Error::NoEntry) => {
                        let fallback = self.fallback_store.lock().map_err(|_| {
                            ApplicationError::PersistenceError("Failed to acquire credential lock".into())
                        })?;
                        Ok(fallback.get(account_identifier).cloned())
                    }
                    Err(_) => {
                        let fallback = self.fallback_store.lock().map_err(|_| {
                            ApplicationError::PersistenceError("Failed to acquire credential lock".into())
                        })?;
                        Ok(fallback.get(account_identifier).cloned())
                    }
                }
            }
            Err(_) => {
                let fallback = self.fallback_store.lock().map_err(|_| {
                    ApplicationError::PersistenceError("Failed to acquire credential lock".into())
                })?;
                Ok(fallback.get(account_identifier).cloned())
            }
        }
    }

    /// Deletes a stored credential upon account disconnection.
    pub fn delete_token(&self, account_identifier: &str) -> Result<(), ApplicationError> {
        let entry = Entry::new(&self.service_name, account_identifier);
        if let Ok(keyring_entry) = entry {
            let _ = keyring_entry.delete_credential();
        }

        let mut fallback = self.fallback_store.lock().map_err(|_| {
            ApplicationError::PersistenceError("Failed to acquire credential lock".into())
        })?;
        fallback.remove(account_identifier);
        Ok(())
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_store_fallback_operations() {
        let store = CredentialStore::with_service_name("gitverse-test");
        let account_id = "test-github-user-123";
        let token = "ghp_secure_dummy_token_123456789";

        // Store credential
        assert!(store.store_token(account_id, token).is_ok());

        // Retrieve credential
        let retrieved = store.get_token(account_id).expect("Should retrieve token");
        assert_eq!(retrieved, Some(token.to_string()));

        // Delete credential
        assert!(store.delete_token(account_id).is_ok());

        // Verify gone
        let after_delete = store.get_token(account_id).expect("Should check token");
        assert_eq!(after_delete, None);
    }
}
