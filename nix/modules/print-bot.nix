{ config, lib, pkgs, ... }:

let
  cfg = config.services.print-bot;

  print-bot = pkgs.callPackage ../pkgs/print-bot.nix { };
in {
  options.services.print-bot = {
    enable = lib.mkEnableOption "Print Bot service";

    secretsFile = lib.mkOption {
      type = lib.types.path;
      description = "Path to environment file containing TELOXIDE_TOKEN and ADMIN_GROUP_ID";
    };

    dataDir = lib.mkOption {
      type = lib.types.str;
      default = "/var/lib/print-bot";
      description = "Directory for storing uploaded PDFs";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "print-bot";
      description = "User to run the service as";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "print-bot";
      description = "Group to run the service as";
    };
  };

  config = lib.mkIf cfg.enable {
    users.users.${cfg.user} = lib.mkIf (cfg.user == "print-bot") {
      isSystemUser = true;
      group = cfg.group;
      extraGroups = [ "lp" "keys" ];
    };

    users.groups.${cfg.group} = lib.mkIf (cfg.group == "print-bot") { };

    systemd.tmpfiles.rules = [
      "d ${cfg.dataDir} 0755 ${cfg.user} ${cfg.group} -"
      "d ${cfg.dataDir}/uploads 0755 ${cfg.user} ${cfg.group} -"
    ];

    systemd.services.print-bot = {
      description = "Print Bot";
      after = [ "network-online.target" "cups.service" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      path = [ pkgs.poppler_utils pkgs.cups ];

      environment = {
        RUST_LOG = "info";
      };

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        WorkingDirectory = cfg.dataDir;
        ExecStart = "${print-bot}/bin/rust_test_bot";
        EnvironmentFile = cfg.secretsFile;
        Restart = "always";
        RestartSec = 5;

        # Hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.dataDir ];
        PrivateTmp = true;
        SupplementaryGroups = [ "lp" ];
      };
    };
  };
}
