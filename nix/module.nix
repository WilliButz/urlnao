{ config, lib, pkgs, ... }:
let
  inherit (lib.strings)
    escapeShellArg
    concatStringsSep;

  inherit (lib.options)
    types
    mkOption
    mkEnableOption;

  inherit (lib.modules)
    mkDefault
    mkIf;

  inherit (lib.types)
    enum
    listOf
    str;

  cfg = config.custom.services.urlnao;
in
{
  options.custom.services.urlnao = {
    enable = mkEnableOption "the urlnao service";

    hostname = mkOption {
      type = str;
      description = ''
      '';
    };

    protocol = mkOption {
      type = enum [ "http" "https" ];
      default = "https";
      description = ''
      '';
    };

    extraArgs = mkOption {
      type = listOf str;
      default = [];
      description = ''
      '';
    };
  };

  config = mkIf cfg.enable {
    users.users.urlnao = {
      isSystemUser = true;
      group = "urlnao";
      description = "urlnao service user";
    };
    users.groups.urlnao = {};

    services.nginx = {
      enable = mkDefault true;
      recommendedOptimisation = mkDefault true;
      recommendedProxySettings = mkDefault true;
      recommendedTlsSettings = mkDefault true;
      virtualHosts.${cfg.hostname} = {
        enableACME = cfg.protocol == "https";
        forceSSL = cfg.protocol == "https";
        locations."/".proxyPass = "http://unix:/var/lib/urlnao/urlnao.sock";
      };
    };

    systemd.services.nginx.serviceConfig = {
      SupplementaryGroups = [ "urlnao" ];
    };

    systemd.services.urlnao = {
      wantedBy = [ "multi-user.target" ];
      before = [ "nginx.service" ];
      serviceConfig = {
        CapabilityBoundingSet = "";
        DevicePolicy = "closed";
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProcSubset = "pid";
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = "full";
        RemoveIPC = true;
        RestrictAddressFamilies = "AF_UNIX";
        RestrictNamespaces = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        Restart = "always";
        UMask = "0077";
        Group = "urlnao";
        User = "urlnao";
        StateDirectory = "urlnao";
        StateDirectoryMode = "0710";
        WorkingDirectory = "/var/lib/urlnao";
        ExecStart = concatStringsSep " " ([
          "@${pkgs.urlnao}/bin/urlnao urlnao"
          "--hostname ${escapeShellArg cfg.hostname}"
          "--protocol ${escapeShellArg cfg.protocol}"
        ] ++ (map escapeShellArg cfg.extraArgs));
      };
    };
  };
}
