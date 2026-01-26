![](https://github.com/trvtsn/pkhk_ctf/blob/dev/site_introduction.gif)

# PKHK CTF Platform

Early prototype of the CTF platform for [Pärnumaa Kutsehariduskeskus](https://hariduskeskus.ee/).
Full-stack project using Leptos SSR, Axum and Tailwind v4.

## Running the project

1. Install rust & cargo using [rustup](https://rustup.rs/)
2. Install [cargo-leptos](https://github.com/leptos-rs/cargo-leptos) (*Recommended to install with [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) on Windows due to headaches relating to openssl libraries*)

```bash
cargo leptos watch
```

## Updates

### 11. January 2026

- Project is nearing "beta" status. With the core functionalities and site structure having been implemented, we can begin placing our focus on additional UI/UX design along with the identification and mitigation of any potential bugs and performance optimization. Most features have already been implemented, so any work relating to new feature additions can be, for now, safely set aside.

## To-Do List

### Functionality
- [ ] Mitigate any unnecessary hydration/data-processing loops or cycles (to save resources)
- [ ] Clean up code for better readability, get rid of placeholder code
- [ ] Properly handle all errors, remove all .unwrap()'s where necessary
- [x] Actually make dark mode work
- [x] Add "group" column to DB in "users" table
- [ ] Filter challenges and leaderboard by user group
- [x] Add "Group" select to admin create challenge view
- [ ] Add LDAP/AD functionality

### Security

### Styling
- [ ] Create fitting style for dark mode
- [ ] Remove flag input box on challenge solve
- [ ] Fix tooltip color on leaderboard hover
- [ ] Fix select option color

### Project Longevity
- [ ] Add doc comments to pages and components
