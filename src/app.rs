use crate::components::{MessageContext, MessageSeverity, Messages};
use crate::github::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use server_fn::error::NoCustomError;
use std::sync::Arc;

const GITHUB_CLIENT_ID: &str = "Ov23lixO0S9pamhwo1u7";

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

    let client = reqwest::Client::new();
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", GITHUB_CLIENT_ID),
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
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri=http://127.0.0.1:8787/oauth/callback&scope=read:project read:org",
        GITHUB_CLIENT_ID
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
            spawn_local(async move {
                let response = token.user_repositories().await;

                if let Ok(repositories) = response {
                    set_repos.set(repositories);
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
        if let Some(token) = get_access_token_from_storage() {
            spawn_local(async move {
                let login = token.user().await.map(|user| user.login).ok();

                if let Some(login) = login {
                    // Fetch organizations using the login name
                    let response = token.organizations(&login).await;

                    if let Ok(organizations) = response {
                        set_orgs.set(organizations.clone());

                        // Fetch repositories for each organization
                        let mut org_repositories = std::collections::HashMap::new();
                        for org in organizations {
                            let repos_response = token.org_repositories(&org.login).await;

                            if let Ok(repositories) = repos_response {
                                org_repositories.insert(org.login, repositories);
                            }
                        }
                        set_org_repos.set(org_repositories);
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

    // Add the message context
    let message_ctx = MessageContext::new();
    provide_context(message_ctx);

    view! {
        <Body class="bg-sky-100" />

        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta http-equiv="content-type" content="text/html; charset=utf-8" />
        <Stylesheet href="/style.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />

        <Messages/>

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

#[derive(Params, Clone, Debug, PartialEq, Eq)]
struct OAuthCallbackParams {
    code: Option<String>,
}

#[component]
fn OAuthCallback() -> impl IntoView {
    let navigate = use_navigate();
    let params = use_query::<OAuthCallbackParams>();
    let message_ctx = expect_context::<MessageContext>();

    create_effect(move |_| {
        let navigate = navigate.clone();
        let message_ctx = message_ctx.clone();

        if let Ok(OAuthCallbackParams { code: Some(code) }) = params.get() {
            spawn_local(async move {
                match exchange_token(code).await {
                    Ok(token) => {
                        store_access_token(&token);
                        message_ctx.add("Successfully logged in!", MessageSeverity::Info);
                        navigate("/", NavigateOptions::default());
                    }
                    Err(e) => {
                        message_ctx.add(format!("Failed to login: {}", e), MessageSeverity::Error);
                        navigate("/", NavigateOptions::default());
                    }
                }
            });
        }
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

fn get_access_token_from_storage() -> Option<UserAccessToken> {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|storage| storage.get_item("github_token").ok().flatten())
        .map(|token| UserAccessToken::from_string(token))
}
