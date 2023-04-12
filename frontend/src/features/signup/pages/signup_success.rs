use yew::prelude::*;

#[function_component(SignupSuccess)]
pub fn signup_success() -> Html {
    html! {
        <div class="centered-box">
           <h1 class="centered-box__title">{"Create account"}</h1>
            <div>
                {"You received a email with a registration validation link. Please click on it before logging in."}
            </div>
        </div>
    }
}
