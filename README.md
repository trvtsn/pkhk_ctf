![](https://github.com/trvtsn/pkhk_ctf/blob/dev/site_introduction.gif)

# PKHK CTF Platform

Very early prototype of the CTF platform for [Pärnumaa Kutsehariduskeskus](https://hariduskeskus.ee/).
Full-stack project using Leptos SSR & Axum.

Focus lies on functionality first, then styling and visual acuity.

## Running the project

1. Install rust & cargo using [rustup](https://rustup.rs/)
2. Install [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) (*Recommended to install with [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) on Windows due to headaches relating to openssl libraries*)

```bash
cargo leptos watch
```

## To-Do List

### Functionality
- [ ] Replace SQLx crate and MySQL DB with SurrealDB
- [ ] Mitigate any unnecessary hydration/data-processing loops or cycles (to save resources)
- [ ] Server-side flag checking
- [ ] Format log messages in Admin "Log" section

### Security
- [ ] Restrict access to admin endpoints and API
- [ ] Hashing of flags
- [ ] A lot, will fill this in later

### Styling
- [ ] A lot, will fill this in later
