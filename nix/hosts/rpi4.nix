{ config, pkgs, lib, ... }:

{
  imports = [ ../modules/print-bot.nix ];

  # Boot configuration for Raspberry Pi 4
  boot = {
    kernelPackages = pkgs.linuxKernel.packages.linux_rpi4;
    initrd.availableKernelModules = [ "xhci_pci" "usbhid" "usb_storage" ];
    loader = {
      grub.enable = false;
      generic-extlinux-compatible.enable = true;
    };
    extraModprobeConfig = "options cfg80211 ieee80211_regdom=PL";
  };

  # Filesystem
  fileSystems."/" = {
    device = "/dev/disk/by-label/NIXOS_SD";
    fsType = "ext4";
    options = [ "noatime" ];
  };

  # Networking
  networking = {
    hostName = "print-bot";
    nameservers = [ "8.8.8.8" "1.1.1.1" ];
    interfaces.eth0.ipv4.addresses = [{
      address = "10.30.30.30";
      prefixLength = 24;
    }];
    firewall.interfaces.eth0.allowedUDPPorts = [ 67 68 ];
  };

  # Print Bot service
  services.print-bot = {
    enable = true;
    secretsFile = "/run/keys/bot-env";
  };

  # DHCP server on eth0 for the printer
  services.dnsmasq = {
    enable = true;
    settings = {
      interface = "eth0";
      bind-interfaces = true;
      dhcp-range = "10.30.30.100,10.30.30.200,24h";
      dhcp-option = [ "option:router,10.30.30.30" ];
    };
  };

  # CUPS printing
  services.printing = {
    enable = true;
    listenAddresses = [ "*:631" ];
    allowFrom = [ "all" ];
    browsing = true;
    defaultShared = true;
    openFirewall = true;
    extraConf = "ServerAlias *";
  };

  # System services
  services = {
    openssh = {
      enable = true;
      settings.PasswordAuthentication = false;
    };
    tailscale = {
      enable = true;
      useRoutingFeatures = "both";
    };
  };

  # User configuration
  users = {
    mutableUsers = false;
    users.print-bot-user = {
      isNormalUser = true;
      extraGroups = [ "wheel" "lp" ];
    };
  };

  # Auto-login on console
  services.getty.autologinUser = "print-bot-user";

  # Packages
  environment.systemPackages = with pkgs; [
    vim
    htop
    git
  ];

  # Hardware
  hardware.enableRedistributableFirmware = true;

  # Passwordless sudo for wheel group (needed for Colmena deployments)
  security.sudo.wheelNeedsPassword = false;

  # Nix settings
  nix.settings.experimental-features = [ "nix-command" "flakes" ];

  system.stateVersion = "23.11";
}
