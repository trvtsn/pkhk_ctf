![](https://github.com/trvtsn/pkhk_ctf/blob/master/site_introduction.gif)

# PKHK CTF Platform

Prototype of the CTF platform for [Pärnumaa Kutsehariduskeskus](https://hariduskeskus.ee/).
Full-stack project using Leptos SSR, Axum and Tailwind v4.

## Setup
Refer to the [setup docs](https://github.com/trvtsn/pkhk_ctf/blob/master/docs/INSTALL.md)

## Updates

### 11. January 2026

- Project is nearing "beta" status. With the core functionalities and site structure having been implemented, we can begin placing our focus on additional UI/UX design along with the identification and mitigation of any potential bugs and performance optimization. Most features have already been implemented, so any work relating to new feature additions can be, for now, safely set aside.

### 9. March 2026

- Project has reached "beta" status. Beta test with live users done. A couple of issues popped up, like the application freezing at certain unknown locations in the code, and the inability to login via username (sAMAccountName) with LDAP. I suspect the application freezes due to some code blocking the main thread, not allowing other executions to take place. However, there's also the possibility of lacking hardware resources during the test. The server was hosted on a Proxmox VM (Debian 13.2.0 OS, 1 socket 8 core CPU, 16 GB RAM). We shall see if the freezing issue has been fixed after some adjustments to the code and a boost to the server's resources.

## To-Do List

### Functionality
- [ ] Mitigate any unnecessary hydration/data-processing loops or cycles (to save resources)
- [ ] Clean up code for better readability, get rid of placeholder code
- [ ] Add LDAPS/ work in progress needs certs from DC2
- [ ] Find or implement a new and more configurable chart builder for the leaderboard

### Security
- [ ] Sanitize text inputs
- [ ] Disallow special characters in text inputs where necessary
- [ ] Replace sensitive data storage like plaintext passwords from LDAP and Proxmox configurations in DB to RAM
- [ ] Handle errors carefully, replace all verbose error returns with minimized error messages and log the verbose errors.

### Styling

### Project Longevity
