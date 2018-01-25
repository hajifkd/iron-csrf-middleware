# iron-csrf-middleware

An Iron middleware to add CSRF protection.

This middleware checks CSRF token any POST request not only for form data but also for json data.

# Usage

First, add the following dependencies to `Cargo.toml`:

```toml
iron = "*"
iron-sessionstorage = { version = "*", git = "https://github.com/iron/iron-sessionstorage.git" }
iron-csrf-middleware= { version = "*", git = "https://github.com/hajifkd/iron-csrf-middleware" }
```

Then, insert `CsrfMiddleware` before dynamical handlers:
```rust
extern crate iron;
extern crate iron_csrf_middleware;
extern crate iron_sessionstorage;
extern crate mount;

use iron::prelude::*;
use mount::Mount;
use iron_sessionstorage::SessionStorage;
use iron_sessionstorage::backends::SignedCookieBackend;
use iron_csrf_middleware::CsrfMiddleware;

fn main() {
    // You need to give secret to generate CSRF token
    let csrf_middleware = CsrfMiddleware::new("hogehoge");

    // Some dynamical handlers
    let mut dyn_handlers = Chain::new(...);
    // Set the middleware here
    dyn_handlers.link_before(csrf_middleware);

    // You can combine with the other handlers which does not need
    // CSRF protections like static files like this.
    // (Of course, this is not necessary.)
    let mut mount = Mount::new();
    mount.mount("/", dyn_handlers);
    mount.mount("/static", some_static_handlers);

    // Finally, you have to wrap entire chains by `SessionStorage`.
    // Note that even if you omit `mount` above, you need to have
    // two `Chain`s. In the first, you need to add `CsrfMiddleware`
    // In the second, which is based on the first, you need to add
    // `SessionStorage`.
    let my_secret = b"fugafuga".to_vec();
    let mut chain = Chain::new(mount);
    chain.link_around(SessionStorage::new(SignedCookieBackend::new(my_secret)));

    // Launch!
    Iron::new(chain).http("localhost:3000").unwrap();
}
```

Finally, you have to append an appropriate CSRF token in handlers:

```rust
fn handler_html(req: &mut ::iron::Request) -> IronResult<()> {
    let csrf_token: String = req.csrf_token();
    let query_key = ::iron_csrf_middleware::QUERY_KEY;

    Ok(Response::with((
        ::iron::headers::ContentType::html().0,
        status::Ok,
        format!("<html><body>\
        <form method=POST action=...>\
        <input type=hidden value={} name={} />\
        ...\
        </form>\
        </body></html>", csrf_token, query_key)
    )))
}

fn handler_json(req: &mut ::iron::Request) -> IronResult<()> {
    let csrf_token: String = req.csrf_token();
    let query_key = ::iron_csrf_middleware::QUERY_KEY;

    Ok(Response::with((
        ::iron::headers::ContentType::json().0,
        status::Ok,
        format!("{ \"{}\": \"{}\" }", query_key, csrf_token)
    )))
}
```

For an example, please see [this](https://github.com/hajifkd/iron-diesel-scaffold).