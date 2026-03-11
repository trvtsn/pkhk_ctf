![](https://github.com/trvtsn/pkhk_ctf/blob/dev/site_introduction.gif)

# PKHK CTF Platform

Prototype of the CTF platform for [Pärnumaa Kutsehariduskeskus](https://hariduskeskus.ee/).
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

### 9. March 2026

- Project has reached "beta" status. Beta test with live users done. A couple of issues popped up, like the application freezing at certain unknown locations in the code, and the inability to login via username (sAMAccountName) with LDAP. I suspect the application freezes due to some code blocking the main thread, not allowing other executions to take place. However, there's also the possibility of lacking hardware resources during the test. The server was hosted on a Proxmox VM (Debian 13.2.0 OS, 1 socket 8 core CPU, 16 GB RAM). We shall see if the freezing issue has been fixed after some adjustments to the code and a boost to the server's resources.

## To-Do List

### Functionality
- [ ] Mitigate any unnecessary hydration/data-processing loops or cycles (to save resources)
- [ ] Clean up code for better readability, get rid of placeholder code
- [x] Properly handle all errors, remove all .unwrap()'s where necessary
- [x] Actually make dark mode work
- [x] Add "group" column to DB in "users" table
- [x] Filter challenges and leaderboard by user group
- [x] Add multi-select for "visible to group" on admin event/challenge create/edit
- [x] Add "Group" select to admin create challenge view
- [x] Add LDAP/AD functionality
- [ ] Add LDAPS/ work in progress needs certs from DC2 
- [x] Create CTF Realm to Proxmox
- [x] Create competitor pools, permissions, accesses in Proxmox
- [x] Create Kali Linux Proxmox !!template!! (that means vm is already preconfigured)
- [x] Create script to automate clone & start process in Proxmox
- [x] Create option to attach mulitple VM-s to a challenge
- [x] When user inputs STOP VM (in proxmox use HARD STOP + additional vm remove api call)
- [x] Add start/stop/console permissions to users in their proxmox pools, disallow editing of VM description and other metadata
- [x] Remove VMs from user challenge popup view after solve
- [x] Fix LDAP login auto-refresh
- [x] Fix refresh on start VM
- [x] Fix signals not allowing writing on admin LDAP inputs
- [x] Add hints to challenges
- [ ] Find or implement a new and more configurable chart builder for the leaderboard
- [x] Allow formatting of challenge descriptions (newline support)
- [x] Allow for multiple toast messages to appear instead of one at a time

### Security
- [ ] Sanitize text inputs
- [ ] Disallow special characters in text inputs where necessary
- [ ] Replace sensitive data storage like plaintext passwords from LDAP and Proxmox configurations in DB to RAM

### Styling
- [x] Create fitting style for dark mode
- [x] Remove flag input box on challenge solve
- [x] Fix tooltip color on leaderboard hover
- [x] Fix select option color
- [x] Add spinner for indicating pending tasks
- [x] Add toast messages

### Project Longevity
- [ ] Add doc comments to pages and components
- [ ] Add setup docs
