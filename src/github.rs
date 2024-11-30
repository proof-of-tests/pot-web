use serde::{Deserialize, Serialize};

// Wish I could use `octocrab` but it doesn't support WASM.
#[derive(Clone, Debug, Deserialize)]
pub struct Repository {
    // pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub private: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Organization {
    pub login: String,
    pub avatar_url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub login: String,
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
            "url": "https://api.github.com/users/octocat"
        }"#;

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.login, "octocat");
    }

    // Test that User can be deserialized from a real GitHub API response
    #[test]
    fn user_json_unit_test_2() {
        let json = include_str!("../tests/user.json");
        let user: User = serde_json::from_str(json).unwrap();
        assert!(user.login.len() > 0);
    }
}
