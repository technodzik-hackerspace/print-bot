let
  nodeName = "rpi4";
in
{
  meta = {
    nixpkgs = import (builtins.fetchGit {
      name = "nixos-24.11-2025-01-15";
      url = "https://github.com/NixOS/nixpkgs";
      ref = "refs/heads/nixos-24.11";
    }) {};
  };

  ${nodeName} = { lib, name, ... }: let
    secretsFile = ./secrets/secrets.nix;
    secrets = if builtins.pathExists secretsFile then import secretsFile else {};
    get = path: lib.attrByPath path null secrets;
  in {
    imports = [ ./hosts/rpi4.nix ];

    deployment = {
      targetHost = lib.mkForce (get [ "hosts" name "sshHost" ]);
      targetUser = lib.mkForce (get [ "hosts" name "sshUser" ]);
      buildOnTarget = true;

      keys = {
        "bot-env" = {
          text = get [ "hosts" name "botEnv" ];
          destDir = "/run/keys";
          permissions = "0640";
          group = "print-bot";
        };
      };
    };

    users.users.print-bot-user.openssh.authorizedKeys.keys = [
      (get [ "hosts" name "sshKey" ])
    ];

    hardware.printers = {
      ensureDefaultPrinter = "td-printer";
      ensurePrinters = [{
        name = "td-printer";
        description = "HP LaserJet M207-M212";
        deviceUri = get [ "hosts" name "printerUri" ];
        model = "everywhere";
      }];
    };

    services.dnsmasq.settings.dhcp-host =
      "${get [ "hosts" name "printerMac" ]},td-printer,10.30.30.66";

    networking.wireless = {
      enable = true;
      networks.${get [ "hosts" name "wifiSSID" ]}.psk = get [ "hosts" name "wifiPassword" ];
      interfaces = [ "wlan0" ];
    };

    assertions = [
      {
        assertion = get [ "hosts" name "sshHost" ] != null;
        message = "Missing sshHost for '${name}'. Create secrets/secrets.nix";
      }
    ];

    nixpkgs.system = "aarch64-linux";
  };
}
