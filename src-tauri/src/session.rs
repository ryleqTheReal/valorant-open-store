// ======= START SESSION =======

use std::sync::RwLock;

use crate::types::{DiscordProfile, WebTokenPair};

pub struct Session {
    token_pair: RwLock<Option<WebTokenPair>>,
    profile: RwLock<Option<DiscordProfile>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            token_pair: RwLock::new(None),
            profile: RwLock::new(None),
        }
    }

    pub fn set(&self, token_pair: WebTokenPair, profile: DiscordProfile) {
        *self.token_pair.write().unwrap() = Some(token_pair);
        *self.profile.write().unwrap() = Some(profile);
    }

    pub fn access_token(&self) -> Option<String> {
        self.token_pair
            .read()
            .unwrap()
            .as_ref()
            .map(|t| t.access_token.clone())
    }

    #[allow(dead_code)]
    pub fn profile(&self) -> Option<DiscordProfile> {
        self.profile.read().unwrap().clone()
    }

    pub fn logout(&self) {
        *self.token_pair.write().unwrap() = None;
        *self.profile.write().unwrap() = None;
    }

    #[allow(dead_code)]
    pub fn is_logged_in(&self) -> bool {
        self.token_pair.read().unwrap().is_some()
    }
}

// ======= END SESSION =======

// ======= START TESTS =======

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pair() -> WebTokenPair {
        WebTokenPair {
            access_token: "at".into(),
            access_token_expires_at: 4099680000,
            refresh_token: "rt".into(),
            refresh_token_expires_at: 4099680000,
        }
    }

    fn make_profile() -> DiscordProfile {
        DiscordProfile {
            id: "123".into(),
            username: "user".into(),
            avatar_url: "https://example.com/avatar.png".into(),
        }
    }

    #[test]
    fn starts_not_logged_in() {
        let s = Session::new();
        assert!(!s.is_logged_in());
        assert!(s.access_token().is_none());
        assert!(s.profile().is_none());
    }

    #[test]
    fn set_makes_logged_in() {
        let s = Session::new();
        s.set(make_pair(), make_profile());
        assert!(s.is_logged_in());
        assert_eq!(s.access_token().as_deref(), Some("at"));
        assert_eq!(s.profile().map(|p| p.id).as_deref(), Some("123"));
    }

    #[test]
    fn logout_clears_session() {
        let s = Session::new();
        s.set(make_pair(), make_profile());
        s.logout();
        assert!(!s.is_logged_in());
        assert!(s.access_token().is_none());
        assert!(s.profile().is_none());
    }
}

// ======= END TESTS =======
