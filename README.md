# tuinance

Tuinance is a performant tool to display financial data, written completely in Rust.

All data is gathered through the Yahoo Finance API.

### Installation
No binaries are being provided yet, so building from source is the only viable installation method.

Build dependencies are `cargo` and `rustc`.

Build tuinance from source:
```
git clone https://github.com/landchad/tuinance
cd tuinance
cargo install --path .
```

Make sure the output directory is in your $PATH.

You should now be able to run Tuinance via the `tuinance` command.

### Configuration
Tuinance is configured through a configuration file located at `~/.config/tuinance.toml`.

A default configuration file would look something like this:

```toml
# ~/.config/tuinance.toml
tickers = ["FB", "AMZN", "AAPL", "NFLX", "GOOG"]

```
---
## Default Keybinds


```
General
q | Exit

Navigation
h | Decrease the current interval by one
j | Move down in the ticker list
k | Move up in the ticker list
l | Increase the current interval by one

UI
v | Display volume chart instead of price chart
z | Display chart in fullscreen

```
## Preview

![Preview](media/preview.png?raw=true "Preview")
