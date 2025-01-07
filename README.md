# tmux-popr

## Features

- Create persistent popup sessions
- Sessions run on an isolated socket so they don't muddy up "main" process
  sessions.
- Popups are unique to a host session, for context isolation.
- Run commands in popup sessions (e.g., lazygit)

## Installation

### Using Nix Flakes

Add to your flake inputs:
```nix
{
  inputs.tmux-popr.url = "github:addisonbeck/tmux-popr";
}
```

### Using Cargo

```bash
cargo install --git https://github.com/username/tmux-popr
```

## Usage

In your tmux config:
```tmux
# Toggle a persistent popup session
bind-key -n C-p run-shell "tmux-popr"

# Toggle lazygit in a popup
bind-key -n C-l run-shell "tmux-popr lazygit"
```

Run any command in a popup:
```bash
tmux-popr <command>
```
