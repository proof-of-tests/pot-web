use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use serde::Deserialize;
use serde::Serialize;
use server_fn::error::NoCustomError;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    html_url: String,
    private: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct Organization {
    login: String,
    avatar_url: String,
}

#[derive(Clone, Debug, Deserialize)]
struct User {
    login: String,
}

#[derive(Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    scope: String,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[server(ExchangeToken, "/api")]
#[worker::send]
pub async fn exchange_token(code: String) -> Result<String, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use worker::Env;

    let Extension(env): Extension<Arc<Env>> = extract().await?;
    let client_secret = env
        .secret("GITHUB_CLIENT_SECRET")
        .map_err(|_| ServerFnError::ServerError::<NoCustomError>("Missing GITHUB_CLIENT_SECRET".into()))?
        .to_string();
    let client_id = "Ov23lixO0S9pamhwo1u7";

    let client = reqwest::Client::new();
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("client_secret", &client_secret),
            ("code", &code),
        ])
        .send()
        .await
        .map_err(|e| ServerFnError::ServerError::<NoCustomError>(e.to_string()))?;

    if response.status().is_success() {
        let token_response = response
            .json::<TokenResponse>()
            .await
            .map_err(|e| ServerFnError::ServerError::<NoCustomError>(e.to_string()))?;
        Ok(token_response.access_token)
    } else {
        let error = response
            .json::<ErrorResponse>()
            .await
            .map_err(|e| ServerFnError::ServerError::<NoCustomError>(e.to_string()))?;
        Err(ServerFnError::ServerError::<NoCustomError>(error.error))
    }
}

#[component]
fn LoginButton() -> impl IntoView {
    let client_id = "Ov23lixO0S9pamhwo1u7";
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri=http://127.0.0.1:8787/oauth/callback&scope=read:project read:org",
        client_id
    );

    view! {
        <a
            href=auth_url
            class="inline-block px-4 py-2 bg-gray-900 text-white rounded hover:bg-gray-700 transition-colors"
        >
            "Login with GitHub"
        </a>
    }
}

#[component]
fn RepositoryList() -> impl IntoView {
    let (repos, set_repos) = create_signal(Vec::<Repository>::new());

    create_effect(move |_| {
        // This would normally come from your OAuth flow
        let access_token = get_access_token_from_storage();
        if let Some(token) = access_token {
            let client = reqwest::Client::new();
            let set_repos = set_repos.clone();
            spawn_local(async move {
                let response = client
                    .get("https://api.github.com/user/repos")
                    .header("Authorization", format!("Bearer {}", token))
                    .header("User-Agent", "proof-of-tests")
                    .send()
                    .await;

                if let Ok(response) = response {
                    if let Ok(repositories) = response.json::<Vec<Repository>>().await {
                        set_repos.set(repositories);
                    }
                }
            });
        }
    });

    view! {
        <div class="space-y-4">
            <h2 class="text-2xl font-bold">"Your Repositories"</h2>
            <div class="space-y-2">
                {move || repos.get().into_iter().map(|repo| {
                    view! {
                        <div class="p-4 border rounded hover:bg-gray-50">
                            <a href=repo.html_url target="_blank" class="font-medium hover:underline">
                                {repo.full_name}
                            </a>
                            <span class="ml-2 text-sm text-gray-500">
                                {if repo.private { "Private" } else { "Public" }}
                            </span>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn OrganizationList() -> impl IntoView {
    let (orgs, set_orgs) = create_signal(Vec::<Organization>::new());
    let (org_repos, set_org_repos) = create_signal(std::collections::HashMap::<String, Vec<Repository>>::new());

    create_effect(move |_| {
        let access_token = get_access_token_from_storage();
        if let Some(token) = access_token {
            let client = reqwest::Client::new();
            let set_orgs = set_orgs.clone();
            let set_org_repos = set_org_repos.clone();

            spawn_local(async move {
                // First fetch user info to get login name
                let user_response = client
                    .get("https://api.github.com/user")
                    .header("Authorization", format!("Bearer {}", token))
                    .header("User-Agent", "proof-of-tests")
                    .send()
                    .await;

                let login = match user_response {
                    Ok(response) => {
                        if let Ok(user) = response.json::<User>().await {
                            Some(user.login)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                };

                if let Some(login) = login {
                    // Fetch organizations using the login name
                    let response = client
                        .get(format!("https://api.github.com/users/{}/orgs", login))
                        .header("Authorization", format!("Bearer {}", token))
                        .header("User-Agent", "proof-of-tests")
                        .send()
                        .await;

                    if let Ok(response) = response {
                        if let Ok(organizations) = response.json::<Vec<Organization>>().await {
                            set_orgs.set(organizations.clone());

                            // Fetch repositories for each organization
                            let mut org_repositories = std::collections::HashMap::new();
                            for org in organizations {
                                let repos_response = client
                                    .get(format!("https://api.github.com/orgs/{}/repos", org.login))
                                    .header("Authorization", format!("Bearer {}", token))
                                    .header("User-Agent", "proof-of-tests")
                                    .send()
                                    .await;

                                if let Ok(repos_response) = repos_response {
                                    if let Ok(repositories) = repos_response.json::<Vec<Repository>>().await {
                                        org_repositories.insert(org.login, repositories);
                                    }
                                }
                            }
                            set_org_repos.set(org_repositories);
                        }
                    }
                }
            });
        }
    });

    view! {
        <div class="space-y-4">
            <h2 class="text-2xl font-bold">"Your Organizations"</h2>
            <div class="space-y-6">
                {move || orgs.get().into_iter().map(|org| {
                    let org_repositories = org_repos.get().get(&org.login).cloned().unwrap_or_default();
                    view! {
                        <div class="space-y-2">
                            <div class="flex items-center space-x-2">
                                <img src=org.avatar_url class="w-8 h-8 rounded-full" />
                                <h3 class="text-xl font-semibold">{org.login}</h3>
                            </div>
                            <div class="ml-10 space-y-2">
                                {org_repositories.into_iter().map(|repo| {
                                    view! {
                                        <div class="p-4 border rounded hover:bg-gray-50">
                                            <a href=repo.html_url target="_blank" class="font-medium hover:underline">
                                                {repo.full_name}
                                            </a>
                                            <span class="ml-2 text-sm text-gray-500">
                                                {if repo.private { "Private" } else { "Public" }}
                                            </span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Body class="bg-sky-100" />

        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta http-equiv="content-type" content="text/html; charset=utf-8" />
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <div class="w-full h-128 bg-gradient-to-b from-sky-700 from-30% to-sky-100"></div>

        <h1 class="text-6xl font-bold text-center pt-6 mb-2 -mt-128 text-white">
            "Proof of Tests"
        </h1>

        <div class="max-w-4xl mx-auto p-4 bg-white" style:box-shadow="0 0px 5px rgba(0, 0, 0, 0.4)">
            <Router>
                <main>
                    <Routes>
                        <Route
                            path="/"
                            view=move || {
                                view! {
                                    <div class="space-y-8">
                                        <div class="text-center">
                                            <LoginButton/>
                                        </div>
                                        <RepositoryList/>
                                        <OrganizationList/>
                                    </div>
                                }
                            }
                        />
                        <Route
                            path="/oauth/callback"
                            view=move || {
                                view! {
                                    <OAuthCallback/>
                                }
                            }
                        />
                    </Routes>
                </main>
            </Router>
        </div>
    }
}

#[component]
fn OAuthCallback() -> impl IntoView {
    let navigate = use_navigate();

    create_effect(move |_| {
        let navigate = navigate.clone();
        spawn_local(async move {
            let query_string = window().location().search().unwrap_or_default();
            let code = url::Url::parse(&format!("http://dummy{}", query_string))
                .ok()
                .and_then(|url| {
                    url.query_pairs()
                        .find(|(key, _)| key == "code")
                        .map(|(_, value)| value.to_string())
                });

            if let Some(code) = code {
                match exchange_token(code).await {
                    Ok(token) => {
                        store_access_token(&token);
                        navigate("/", NavigateOptions::default());
                    }
                    Err(e) => {
                        log::error!("Failed to exchange token: {:?}", e);
                        // You might want to navigate to an error page here
                        navigate("/", NavigateOptions::default());
                    }
                }
            }
        });
    });

    view! {
        <div class="text-center">
            "Processing login..."
        </div>
    }
}

fn store_access_token(token: &str) {
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.set_item("github_token", token);
    }
}

fn get_access_token_from_storage() -> Option<String> {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|storage| storage.get_item("github_token").ok().flatten())
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

        assert_eq!(repositories[0].name, "repo1");
        assert_eq!(repositories[0].full_name, "user/repo1");
        assert_eq!(repositories[0].html_url, "https://github.com/user/repo1");
        assert_eq!(repositories[0].private, false);

        assert_eq!(repositories[1].name, "repo2");
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
