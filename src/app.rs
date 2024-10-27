use leptos_meta::*;
use leptos::*;

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

        <h1 class="text-6xl font-bold text-center pt-6 mb-2 -mt-128 text-white">"Erudify Dictionary"</h1>
        <h2 class="text-2xl text-center mb-6 text-white">"Chinese-English-Pinyin"</h2>

        <div class="max-w-4xl mx-auto p-4 bg-white" style:box-shadow="0 0px 5px rgba(0, 0, 0, 0.4)">
            App goes here
        </div>
    }
}
