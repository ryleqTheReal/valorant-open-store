// ======= START CONFIG =======

pub const VW_API: &str = "https://valstats.site";
pub const STORE: &str = "https://store.valstats.site";

pub const RIOT_CLIENT_ID: &str = "ritoplus";
pub const RIOT_AUTHORIZE_URL: &str = "https://auth.riotgames.com/authorize";
pub const RIOT_SCOPE: &str =
    "lol summoner offline_access openid link ban account riot://riot.authenticator/session.auth";
pub const RIOT_REDIRECT_URI: &str = "http://localhost/redirect";
pub const RIOT_TOKEN_URL: &str = "https://auth.riotgames.com/token";
pub const RIOT_ENTITLEMENTS_URL: &str = "https://entitlements.auth.riotgames.com/api/token/v1";
pub const RIOT_PRM: &str = "alias:change game:play riot:interact chat:text chat:voice friend:add";
pub const RIOT_UA: &str =
    "RiotGamesApi/26.3.0.0 rso-auth (Android;15.00;AP4A.250105.002;) ritoplus/5.3.0";

pub fn vw_api() -> String {
    std::env::var("VW_API_BASE_URL").unwrap_or_else(|_| VW_API.to_string())
}

pub fn store() -> String {
    std::env::var("STORE_BASE_URL").unwrap_or_else(|_| STORE.to_string())
}

pub fn signed_in_url() -> String {
    format!("{}/signed-in?open-store", vw_api())
}

// ======= END CONFIG =======

// ======= START TESTS =======

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_in_url_targets_vw_api_signed_in_path() {
        assert_eq!(
            signed_in_url(),
            "https://valstats.site/signed-in?open-store"
        );
    }
}

// ======= END TESTS =======
