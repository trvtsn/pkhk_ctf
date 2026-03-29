# Installation

These commands were tested on a Debian 13.2 VM.

## 1. Install Rust

```bash
sudo apt install build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
cargo install cargo-leptos
```

## 2. Clone repository

```bash
git clone https://github.com/trvtsn/pkhk_ctf
cd pkhk_ctf
```

## 3. Install MySQL

```bash
sudo apt install mariadb-server
mysql -u root -p < ./db_creation_script.sql
```

Then create a database user (Must match what is written in .env):

```bash
mysql
```
```sql
CREATE USER ctfpkhk@localhost IDENTIFIED BY "Reverse5";
GRANT ALL PRIVILEGES ON ctfpkhk.* TO ctfpkhk@localhost;
EXIT;
```

## 4. (Optional) Import premade challenges

```bash
mysql -u root -p ctfpkhk < ./premade_challenges.sql
```

## 5. Add Leptos environment variables to PATH

```bash
echo -e "LEPTOS_SITE_ADDR=0.0.0.0:80\nLEPTOS_SITE_ROOT=/path/to/repo/pkhk_ctf/target/release/site" >> ~/.profile
```

## 6. Compile & Run

```bash
cargo leptos build --release
/path/to/repo/pkhk_ctf/target/release/pkhk_ctf
```
