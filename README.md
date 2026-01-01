![](https://github.com/trvtsn/pkhk_ctf/blob/dev/site_introduction.gif)

# PKHK CTF Platform

Very early prototype of the CTF platform for [Pärnumaa Kutsehariduskeskus](https://hariduskeskus.ee/).
Full-stack project using Leptos SSR, Axum and Tailwind v4.

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
- [x] Create challenge categories
- [ ] Clean up code for better readability, get rid of placeholder code

### Security
- [x] Restrict access to admin endpoints and API
- [ ] Hashing of flags
- [ ] Increase session cookie length and complexity
- [x] Generalize/structurize API endpoint names (e.g. **GET /api/build_leaderboard_data** -> **POST /api/leaderboard** and **GET /api/get_db_user** -> **POST /api/user**)

### Styling
- [x] Display challenges and users in uniform boxes
- [x] Add custom site favicon
- [ ] Add icons
- [x] Add "Points" indicator to user on navbar and profile
- [x] Format log messages in Admin "Log" section
