#![deny(warnings)]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{routing::post, Router};
    use hello_world_axum::app::App;
    use hello_world_axum::fileserv::file_and_error_handler;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use leptos_meta::MetaTags;
    use tokio::net::TcpListener;

    simple_logger::init_with_level(log::Level::Debug).expect("couldn't initialize logging");

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // build our application with a route
    let app = Router::new()
        .route("/api/*fn_name", post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || {
                use leptos::prelude::*;

                view! {
                    <!DOCTYPE html>
                    <html>
                        <head>
                            <meta charset="utf-8"/>
                            <meta name="viewport" content="width=device-width, initial-scale=1"/>
                            <AutoReload options=leptos_options.clone() />
                            <HydrationScripts options=leptos_options.clone()/>
                            <MetaTags/>
                        </head>
                        <body>
                            <App/>
                        </body>
                    </html>
                }
            }
        })
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    leptos::logging::log!("listening on http://{}", &addr);
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
#[cfg(not(feature = "ssr"))]
fn main() {}
