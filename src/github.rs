use serde::{Deserialize, Serialize};

// Wish I could use `octocrab` but it doesn't support WASM.
#[derive(Clone, Debug, Deserialize)]
pub struct Repository {
    // pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub private: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct Organization {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct User {
    pub login: String,
    pub avatar_url: String,
    pub gravatar_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct UserAccessToken {
    pub access_token: String,
}

impl UserAccessToken {
    pub fn from_string(s: String) -> Self {
        Self { access_token: s }
    }

    pub async fn user(&self) -> Result<User, reqwest::Error> {
        let client = reqwest::Client::new();

        // First fetch user info to get login name
        let user_response = client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "proof-of-tests")
            .send()
            .await?;

        user_response.json::<User>().await
    }

    pub async fn organizations(&self, login: &str) -> Result<Vec<Organization>, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("https://api.github.com/users/{}/orgs", login))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "proof-of-tests")
            .send()
            .await?;

        response.json::<Vec<Organization>>().await
    }

    pub async fn org_repositories(&self, login: &str) -> Result<Vec<Repository>, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("https://api.github.com/orgs/{}/repos", login))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "proof-of-tests")
            .send()
            .await?;

        response.json::<Vec<Repository>>().await
    }

    pub async fn user_repositories(&self) -> Result<Vec<Repository>, reqwest::Error> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.github.com/user/repos")
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", "proof-of-tests")
            .send()
            .await?;

        response.json::<Vec<Repository>>().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sanity check that `Repository` can be deserialized from JSON
    #[test]
    fn repository_json_unit_test_1() {
        let json = r#"[
            {
                "name": "repo1",
                "full_name": "user/repo1",
                "html_url": "https://github.com/user/repo1",
                "private": false
            },
            {
                "name": "repo2",
                "full_name": "user/repo2",
                "html_url": "https://github.com/user/repo2",
                "private": true
            }
        ]"#;

        let repositories: Vec<Repository> = serde_json::from_str(json).unwrap();

        assert_eq!(repositories.len(), 2);

        // assert_eq!(repositories[0].name, "repo1");
        assert_eq!(repositories[0].full_name, "user/repo1");
        assert_eq!(repositories[0].html_url, "https://github.com/user/repo1");
        assert_eq!(repositories[0].private, false);

        // assert_eq!(repositories[1].name, "repo2");
        assert_eq!(repositories[1].full_name, "user/repo2");
        assert_eq!(repositories[1].html_url, "https://github.com/user/repo2");
        assert_eq!(repositories[1].private, true);
    }

    // Verify that `Repository` can be deserialized from a real GitHub API response
    #[test]
    fn repository_json_unit_test_2() {
        let json = include_str!("../tests/user-repos.json");
        let repositories: Vec<Repository> = serde_json::from_str(json).unwrap();
        assert_eq!(repositories.len(), 30);
    }

    // Verify that `Repository` can be deserialized from a real GitHub API response
    #[test]
    fn repository_json_unit_test_3() {
        let json = include_str!("../tests/org-repos.json");
        let repositories: Vec<Repository> = serde_json::from_str(json).unwrap();
        assert_eq!(repositories.len(), 6);
    }

    // Test that User can be deserialized from a JSON string
    #[test]
    fn user_json_unit_test_1() {
        let json = r#"{
            "login": "octocat",
            "id": 1,
            "node_id": "MDQ6VXNlcjE=",
            "avatar_url": "https://github.com/images/error/octocat_happy.gif",
            "gravatar_id": "",
            "url": "https://api.github.com/users/octocat"
        }"#;

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "octocat");
        assert_eq!(user.avatar_url, "https://github.com/images/error/octocat_happy.gif");
        assert_eq!(user.gravatar_id, "");
    }

    // Test that User can be deserialized from a real GitHub API response
    #[test]
    fn user_json_unit_test_2() {
        let json = include_str!("../tests/user.json");
        let user: User = serde_json::from_str(json).unwrap();
        assert!(user.login.len() > 0);
    }
}
