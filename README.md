# Print Bot

Telegram bot that prints PDF documents. Send a PDF to the bot and it gets printed on the connected printer.

## How it works

1. User sends a PDF document to the Telegram bot
2. Bot downloads the file and saves it to `uploads/`
3. Page count is extracted via `pdfinfo`
4. Document is sent to the printer via `lp`
5. User and admin group receive a confirmation or error message

## Environment variables

| Variable | Description |
|---|---|
| `TELOXIDE_TOKEN` | Telegram Bot API token |
| `ADMIN_GROUP_ID` | Telegram chat ID for admin notifications |
| `RUST_LOG` | Log level (default: `info`) |

## Development

```bash
cargo build
TELOXIDE_TOKEN=... ADMIN_GROUP_ID=... cargo run
```

Requires `pdfinfo` (poppler-utils) and `lp` (CUPS) on the host.

## NixOS Deployment

The bot runs on a Raspberry Pi 4 and is deployed with [Colmena](https://github.com/zhaofengli/colmena).

### Project structure

```
nix/
├── flake.nix                  # Flake with colmena and nixosConfigurations outputs
├── hive.nix                   # Colmena hive (injects secrets into config)
├── hosts/
│   └── rpi4.nix               # Raspberry Pi 4 host configuration
├── modules/
│   └── print-bot.nix          # NixOS service module
├── pkgs/
│   └── print-bot.nix          # Rust package derivation
└── secrets/
    ├── secrets.nix            # Actual secrets (gitignored)
    └── secrets.nix.example    # Template
```

### Setup

1. Copy the secrets template and fill in your values:

```bash
cp nix/secrets/secrets.nix.example nix/secrets/secrets.nix
vim nix/secrets/secrets.nix
```

2. Deploy:

```bash
cd nix
colmena apply --config hive.nix
```

### Infrastructure overview

- **WiFi**: wpa_supplicant on `wlan0`, credentials from secrets
- **Ethernet**: Static IP `10.30.30.30/24`, direct link to printer
- **DHCP server**: dnsmasq on `eth0`, assigns `10.30.30.66` to the printer
- **Printing**: CUPS with IPP, printer configured as `td-printer`
- **Remote access**: Tailscale + SSH (key-only auth)
- **Bot service**: Runs as `print-bot` system user with systemd hardening