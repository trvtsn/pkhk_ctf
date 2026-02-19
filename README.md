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
- [x] Filter challenges and leaderboard by user group
- [x] Add multi-select for "visible to group" on admin event/challenge create/edit
- [x] Add "Group" select to admin create challenge view
- [x] Add LDAP/AD functionality
- [ ] Add LDAPS/ work in progress needs certs from DC2 
- [x] Create CTF Realm to Proxmox
- [x] Create competitor pools, permissions, accesses in Proxmox
- [ ] Create Kali Linux Proxmox !!template!! (that means vm is already preconfigured)
- [x] Create script to automate clone & start process in Proxmox
- [x] Create option to attach mulitple VM-s to a challenge
- [x] When user inputs STOP VM (in proxmox use HARD STOP + additional vm remove api call)
- [x] Add start/stop/console permissions to users in their proxmox pools, disallow editing of VM description and other metadata
- [x] Remove VMs from user challenge popup view after solve
- [x] Fix LDAP login auto-refresh
- [x] Fix refresh on start VM
- [x] Fix signals not allowing writing on admin LDAP inputs
- [x] Add hints to challenges

### Security
- [ ] Sanitize text inputs
- [ ] Disallow special characters in text inputs where necessary
- [ ] Replace sensitive data storage like plaintext passwords from LDAP and Proxmox configurations in DB to RAM

### Styling
- [x] Create fitting style for dark mode
- [x] Remove flag input box on challenge solve
- [ ] Fix tooltip color on leaderboard hover
- [x] Fix select option color
- [x] Add spinner for indicating pending tasks

### Project Longevity
- [ ] Add doc comments to pages and components
