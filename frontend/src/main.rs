mod components;
mod data;
mod features;
mod menu;

use crate::components::protected::Protected;
use crate::components::tags_provider::{Order, TagsProvider};
use crate::features::authentication::pages::login::Login;
use crate::features::authentication::pages::logout::Logout;
use crate::features::authentication::pages::recover_password_form::RecoverPasswordFormHOC;
use crate::features::authentication::pages::recover_password_start::RecoverPasswordStart;
use crate::features::bookmarks::bookmark_provider::BookmarkProvider;
use crate::features::bookmarks::bookmarks_provider::BookmarksProvider;
use crate::features::bookmarks::bookmarks_query::BookmarksQuery;
use crate::features::bookmarks::pages::bookmarks::BookmarksHOC;
use crate::features::bookmarks::pages::create_bookmark::CreateBookmarkHOC;
use crate::features::bookmarks::pages::delete_bookmark::DeleteBookmarkHOC;
use crate::features::bookmarks::pages::edit_bookmark::EditBookmarkHOC;
use crate::features::bookmarks::pages::view_bookmark::ViewBookmarkHOC;
use crate::features::profile::pages::profile::Profile;
use crate::features::profile::pages::validate_email::ValidateEmail;
use crate::features::signup::pages::signup_form::SignupForm;
use crate::features::signup::pages::signup_success::SignupSuccess;
use crate::features::tag_cloud::pages::tag_cloud::TagCloudHOC;
use crate::menu::Menu;
use gloo_net::http::Request;
use rest_api::application::{GetApplicationResult, URL_APPLICATION};
use rest_api::sessions::{CreateSessionResult, URL_SESSIONS_CURRENT};
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_router::prelude::*;

fn main() {
    yew::Renderer::<App>::new().render();
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Index,

    #[at("/bookmarks")]
    Bookmarks,

    #[at("/bookmarks/~search")]
    BookmarksSearch,

    #[at("/bookmarks/~add")]
    AddBookmark,

    #[at("/bookmarks/:id")]
    ViewBookmark { id: i32 },

    #[at("/bookmarks/:id/~delete")]
    DeleteBookmark { id: i32 },

    #[at("/bookmarks/:id/~edit")]
    EditBookmark { id: i32 },

    #[at("/tags")]
    TagCloud,

    #[at("/tools")]
    Tools,

    #[at("/signup")]
    SignupForm,

    #[at("/signup/success")]
    SignupSuccess,

    #[at("/login")]
    Login,

    #[at("/recover-password")]
    RecoverPasswordStart,

    #[at("/recover-password/:id")]
    RecoverPasswordForm { id: String },

    #[at("/email/:uuid/~validate")]
    ValidateEmail { uuid: uuid::Uuid },

    #[at("/profile")]
    Profile,

    #[at("/logout")]
    Logout,

    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, Default)]
struct State {
    username: Option<AttrValue>,
    commit: Option<AttrValue>,
    build_date: Option<AttrValue>,
}

#[function_component(App)]
fn app() -> Html {
    let state = use_state(State::default);

    {
        let state = state.clone();
        use_effect_once(move || {
            let mut new_state = (*state).clone();
            spawn_local(async move {
                if let Some(CreateSessionResult::Success(session)) =
                    CreateSessionResult::from(Request::get(URL_SESSIONS_CURRENT).send().await).await
                {
                    new_state.username = Some(AttrValue::from(session.username));
                }
                if let Some(GetApplicationResult::Success(application)) =
                    GetApplicationResult::from(Request::get(URL_APPLICATION).send().await).await
                {
                    new_state.commit = Some(AttrValue::from(application.commit));
                    new_state.build_date = Some(AttrValue::from(application.build_date));
                }
                state.set(new_state);
            });

            || {}
        });
    }

    let onlogin = {
        let state = state.clone();
        Callback::from(move |username: AttrValue| {
            let mut new_state = (*state).clone();
            new_state.username = Some(username);
            state.set(new_state);
        })
    };
    let onlogout = {
        let state = state.clone();
        Callback::from(move |_: ()| {
            let mut new_state = (*state).clone();
            new_state.username = None;
            state.set(new_state);
        })
    };
    use_navigator();
    html! {
        <>
            <BrowserRouter>
                <Menu username={state.username.clone()} />
                <a
                    class="github-fork-ribbon right-bottom fixed"
                    href="https://github.com/cpollet/rbm"
                    data-ribbon="Fork me on GitHub"
                    title="Fork me on GitHub"
                >
                    {"Fork me on GitHub"}
                </a>
                <div class="content">
                    <Switch<Route> render={
                        let logged_in = state.username.is_some();
                        move |route| match route {
                            Route::Index | Route::Bookmarks => {
                                html! {
                                    <BookmarksProvider>
                                        <TagsProvider>
                                            <BookmarksHOC />
                                        </TagsProvider>
                                    </BookmarksProvider>
                                }
                            }
                            Route::BookmarksSearch => {
                                html! {
                                    <BookmarksQuery>
                                        <BookmarksProvider>
                                            <TagsProvider>
                                                <BookmarksHOC />
                                            </TagsProvider>
                                        </BookmarksProvider>
                                    </BookmarksQuery>
                                }
                            }
                            Route::AddBookmark => {
                                html! {
                                    <Protected {logged_in}>
                                        <TagsProvider>
                                            <CreateBookmarkHOC />
                                        </TagsProvider>
                                    </Protected>
                                }
                            }
                            Route::ViewBookmark { id } => {
                                html! {
                                    <BookmarkProvider {id}>
                                        <ViewBookmarkHOC />
                                    </BookmarkProvider>
                                }
                            }
                            Route::DeleteBookmark { id } => {
                                html! {
                                    <Protected {logged_in}>
                                        <BookmarkProvider {id}>
                                            <DeleteBookmarkHOC />
                                        </BookmarkProvider>
                                    </Protected>
                                }
                            }
                            Route::EditBookmark { id } => {
                                html! {
                                    <Protected {logged_in}>
                                        <BookmarkProvider {id}>
                                            <TagsProvider>
                                                <EditBookmarkHOC />
                                            </TagsProvider>
                                        </BookmarkProvider>
                                    </Protected>
                                }
                            }
                            Route::TagCloud => {
                                html! {
                                    <TagsProvider order={Order::Name}>
                                        <TagCloudHOC />
                                    </TagsProvider>
                                }
                            }
                            Route::Tools => {
                                html! {
                                    <Protected {logged_in}>
                                        {"todo: tools"}
                                    </Protected>
                                }
                            }
                            Route::SignupForm => {
                                html! {
                                    <SignupForm />
                                }
                            }
                            Route::SignupSuccess => {
                                html! {
                                    <SignupSuccess />
                                }
                            }
                            Route::Login => {
                                html! {
                                    <Login {logged_in} onlogin={onlogin.clone()} />
                                }
                            }
                            Route::RecoverPasswordStart => {
                                html! {
                                    <RecoverPasswordStart />
                                }
                            }
                            Route::RecoverPasswordForm { id } => {
                                html! {
                                    <RecoverPasswordFormHOC {id} />
                                }
                            }
                            Route::ValidateEmail { uuid } => {
                                html! {
                                    <ValidateEmail {uuid} />
                                }
                            }
                            Route::Profile => {
                                html! {
                                    <Protected {logged_in}>
                                        <Profile />
                                    </Protected>
                                }
                            }
                            Route::Logout => {
                                html! {
                                    <Logout onlogout={onlogout.clone()} />
                                }
                            }
                            Route::NotFound => {
                                html! {
                                    <h1>{"404 Not Found"}</h1>
                                }
                            }
                        }
                    } />
                </div>
            </BrowserRouter>
            <div class="footer">
                <a
                    href={format!("https://github.com/cpollet/rbm/tree/{}",
                                  state.commit
                                    .as_ref()
                                    .map(AttrValue::to_string)
                                    .unwrap_or_else(|| "master".to_string())
                         )}
                >
                    {state.commit.clone()}
                </a>
                {" · "} {state.build_date.clone()}
            </div>
        </>
    }
}
