// ======= START TYPES =======

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DpopKey {
    pub kty: String,
    pub crv: String,
    pub x: String,
    pub y: String,
    pub d: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: String,
    pub id_token: String,
    pub entitlements_token: String,
    pub expires_at: i64,
}

#[derive(Debug, Serialize)]
pub struct AddAccountRequest {
    pub dpop_key: DpopKey,
    pub tokens: TokenSet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebTokenPair {
    pub access_token: String,
    pub access_token_expires_at: i64,
    pub refresh_token: String,
    pub refresh_token_expires_at: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddAccountResponse {
    pub puuid: String,
    pub shard: String,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccountSummary {
    pub puuid: String,
    pub shard: String,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscordProfile {
    pub id: String,
    pub username: String,
    pub avatar_url: String,
}

// ======= END TYPES =======

// ======= START TESTS =======

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dpop_key_serde_round_trip() {
        let key = DpopKey {
            kty: "EC".into(),
            crv: "P-256".into(),
            x: "abc".into(),
            y: "def".into(),
            d: "ghi".into(),
        };
        let v = serde_json::to_value(&key).unwrap();
        assert_eq!(v["kty"], "EC");
        assert_eq!(v["crv"], "P-256");
        assert_eq!(v["x"], "abc");
        assert_eq!(v["y"], "def");
        assert_eq!(v["d"], "ghi");
        let back: DpopKey = serde_json::from_value(v).unwrap();
        assert_eq!(back.kty, key.kty);
    }

    #[test]
    fn token_set_serde_round_trip() {
        let ts = TokenSet {
            access_token: "at".into(),
            refresh_token: "rt".into(),
            id_token: "it".into(),
            entitlements_token: "et".into(),
            expires_at: 9999,
        };
        let v = serde_json::to_value(&ts).unwrap();
        assert_eq!(v["access_token"], "at");
        assert_eq!(v["expires_at"], 9999);
        let back: TokenSet = serde_json::from_value(v).unwrap();
        assert_eq!(back.expires_at, ts.expires_at);
    }

    #[test]
    fn add_account_request_wire_shape() {
        let req = AddAccountRequest {
            dpop_key: DpopKey {
                kty: "EC".into(),
                crv: "P-256".into(),
                x: "x1".into(),
                y: "y1".into(),
                d: "d1".into(),
            },
            tokens: TokenSet {
                access_token: "at".into(),
                refresh_token: "rt".into(),
                id_token: "it".into(),
                entitlements_token: "et".into(),
                expires_at: 1000,
            },
        };
        let v = serde_json::to_value(&req).unwrap();
        assert!(v["dpop_key"].is_object());
        assert!(v["tokens"].is_object());
        assert_eq!(v["dpop_key"]["kty"], "EC");
        assert_eq!(v["tokens"]["access_token"], "at");
    }

    #[test]
    fn account_summary_parses_optional_fields() {
        let full = json!({
            "puuid": "p1",
            "shard": "na",
            "game_name": "Player",
            "tag_line": "TAG"
        });
        let s: AccountSummary = serde_json::from_value(full).unwrap();
        assert_eq!(s.game_name.as_deref(), Some("Player"));

        let minimal = json!({ "puuid": "p2", "shard": "eu" });
        let s2: AccountSummary = serde_json::from_value(minimal).unwrap();
        assert!(s2.game_name.is_none());
    }
}

// ======= END TESTS =======
