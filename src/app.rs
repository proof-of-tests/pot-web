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
    let client_secret = env.secret("GITHUB_CLIENT_SECRET")?.to_string();

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
        .await?;

    if response.status().is_success() {
        let token_response = response.json::<TokenResponse>().await?;
        Ok(token_response.access_token)
    } else {
        let error = response.json::<ErrorResponse>().await?;
        Err(ServerFnError::ServerError::<NoCustomError>(error.error))
    }
}

#[derive(Clone, Debug)]
pub struct UserContext {
    logged_in: RwSignal<bool>,
    token: RwSignal<Option<String>>,
    user: Resource<Option<String>, Option<User>>,
}

impl UserContext {
    pub fn new() -> Self {
        let token = if cfg!(not(feature = "ssr")) {
            get_token_from_storage()
        } else {
            None
        };

        let logged_in = create_rw_signal(token.is_some());
        let token = create_rw_signal(token);

        let user = create_local_resource(
            move || token.get(),
            |token| async move {
                match token {
                    Some(token) => UserAccessToken::from_string(token).user().await.ok(),
                    None => None,
                }
            },
        );

        Self { logged_in, token, user }
    }

    pub fn login(&self, token: String) {
        set_token_storage(&token);
        self.token.set(Some(token));
        self.logged_in.set(true);
    }

    pub fn logout(&self) {
        remove_token_storage();
        self.token.set(None);
        self.logged_in.set(false);
    }

    pub fn get_token(&self) -> Option<String> {
        self.token.get()
    }

    pub fn is_logged_in(&self) -> bool {
        self.logged_in.get()
    }

    pub fn user(&self) -> Resource<Option<String>, Option<User>> {
        self.user
    }
}

fn set_token_storage(token: &str) {
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.set_item("github_token", token);
    }
}

fn remove_token_storage() {
    if let Some(storage) = window().local_storage().ok().flatten() {
        let _ = storage.remove_item("github_token");
    }
}

fn get_token_from_storage() -> Option<String> {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|storage| storage.get_item("github_token").ok().flatten())
}

#[component]
fn LoginButton() -> impl IntoView {
    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri=http://127.0.0.1:8787/oauth/callback&scope=read:project+read:org",
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
    let repos = create_local_resource(
        || get_access_token_from_storage(),
        |token| async move {
            match token {
                Some(token) => token.user_repositories().await.ok(),
                None => None,
            }
        },
    );

    view! {
        <div class="space-y-4">
            <h2 class="text-2xl font-bold">"Your Repositories"</h2>
            <div class="space-y-2">
                {move || match repos.get() {
                    None => view! { <div>"Loading..."</div> }.into_view(),
                    Some(None) => view! { <div>"Failed to load repositories"</div> }.into_view(),
                    Some(Some(repositories)) => {
                        repositories.into_iter().map(|repo| {
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
                        }).collect_view()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn OrganizationList() -> impl IntoView {
    let user_ctx = expect_context::<UserContext>();

    let org_data = create_local_resource(
        move || (get_access_token_from_storage(), user_ctx.user().get()),
        |(token, user)| async move {
            match (token, user.flatten()) {
                (Some(token), Some(user)) => {
                    let orgs = token.organizations(&user.login).await.ok().unwrap_or_default();
                    let mut org_map = std::collections::HashMap::new();
                    for org in orgs {
                        if let Ok(repositories) = token.org_repositories(&org.login).await {
                            org_map.insert(org, repositories);
                        }
                    }
                    org_map
                }
                _ => Default::default(),
            }
        },
    );

    view! {
        <div class="space-y-4">
            <h2 class="text-2xl font-bold">"Your Organizations"</h2>
            <div class="space-y-6">
                {move || match org_data.get() {
                    None => view! { <div>"Loading organizations..."</div> }.into_view(),
                    Some(org_map) => {
                        org_map.into_iter().map(|(org, repositories)| {
                            view! {
                                <div class="space-y-2">
                                    <div class="flex items-center space-x-2">
                                        <img src=org.avatar_url class="w-8 h-8 rounded-full" />
                                        <h3 class="text-xl font-semibold">{org.login}</h3>
                                    </div>
                                    <div class="ml-10 space-y-2">
                                        {repositories.into_iter().map(|repo| {
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
                                        }).collect_view()}
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let message_ctx = MessageContext::new();
    provide_context(message_ctx);

    let user_ctx = UserContext::new();
    provide_context(user_ctx);

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
    let user_ctx = expect_context::<UserContext>();

    create_effect(move |_| {
        let navigate = navigate.clone();
        let message_ctx = message_ctx.clone();
        let user_ctx = user_ctx.clone();

        if let Ok(OAuthCallbackParams { code: Some(code) }) = params.get() {
            spawn_local(async move {
                match exchange_token(code).await {
                    Ok(token) => {
                        user_ctx.login(token);
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

fn get_access_token_from_storage() -> Option<UserAccessToken> {
    use_context::<UserContext>()
        .and_then(|ctx| ctx.get_token())
        .map(UserAccessToken::from_string)
}
