self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.korrosync;
  inherit
    (lib)
    mkEnableOption
    mkOption
    mkIf
    types
    ;
in {
  options.services.korrosync = {
    enable = mkEnableOption "Korrosync";

    package = mkOption {
      type = types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "korrosync.packages.\${pkgs.stdenv.hostPlatform.system}.default";
      description = "Korrosync package to use.";
    };

    address = mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = "Address the server binds to.";
    };

    port = mkOption {
      type = types.port;
      default = 3000;
      description = "Port the server listens on.";
    };

    dbPath = mkOption {
      type = types.str;
      default = "/var/lib/korrosync/db.redb";
      description = "Path to the redb database file.";
    };

    tls = {
      enable = mkEnableOption "built-in TLS/HTTPS via rustls";

      certPath = mkOption {
        type = types.path;
        default = "/etc/korrosync/tls/cert.pem";
        description = "Path to the TLS certificate file (PEM format).";
      };

      keyPath = mkOption {
        type = types.path;
        default = "/etc/korrosync/tls/key.pem";
        description = "Path to the TLS private key file (PEM format).";
      };
    };

    rateLimit = {
      perSecond = mkOption {
        type = types.ints.positive;
        default = 2;
        description = "Rate limit replenishment rate per second.";
      };

      burstSize = mkOption {
        type = types.ints.positive;
        default = 5;
        description = "Maximum burst size before rate limiting kicks in.";
      };
    };

    user = mkOption {
      type = types.str;
      default = "korrosync";
      description = "System user under which the service runs.";
    };

    group = mkOption {
      type = types.str;
      default = "korrosync";
      description = "System group under which the service runs.";
    };

    dataDir = mkOption {
      type = types.str;
      default = "/var/lib/korrosync";
      description = "Working/state directory for korrosync.";
    };

    extraEnvironment = mkOption {
      type = types.attrsOf types.str;
      default = {};
      description = "Extra environment variables passed to the service verbatim.";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to open the configured port in the firewall.";
    };
  };

  config = mkIf cfg.enable {
    users.users = mkIf (cfg.user == "korrosync") {
      korrosync = {
        isSystemUser = true;
        inherit (cfg) group;
        home = cfg.dataDir;
      };
    };

    users.groups = mkIf (cfg.group == "korrosync") {
      korrosync = {};
    };

    networking.firewall.allowedTCPPorts = mkIf cfg.openFirewall [cfg.port];

    systemd.services.korrosync = {
      description = "Korrosync - KOReader Sync Server";
      wantedBy = ["multi-user.target"];
      after = ["network.target"];

      environment =
        {
          KORROSYNC_DB_PATH = cfg.dbPath;
          KORROSYNC_SERVER_ADDRESS = "${cfg.address}:${toString cfg.port}";
          KORROSYNC_RATE_LIMIT_PER_SECOND = toString cfg.rateLimit.perSecond;
          KORROSYNC_RATE_LIMIT_BURST_SIZE = toString cfg.rateLimit.burstSize;
        }
        // lib.optionalAttrs cfg.tls.enable {
          KORROSYNC_USE_TLS = "true";
          KORROSYNC_CERT_PATH = toString cfg.tls.certPath;
          KORROSYNC_KEY_PATH = toString cfg.tls.keyPath;
        }
        // cfg.extraEnvironment;

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${lib.getExe cfg.package} serve";
        Restart = "on-failure";
        RestartSec = "5s";

        StateDirectory = "korrosync";
        WorkingDirectory = cfg.dataDir;

        # Filesystem
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        PrivateMounts = true;
        ReadWritePaths = [cfg.dataDir];
        UMask = "0077";

        # Kernel
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        ProtectHostname = true;
        ProtectClock = true;
        ProtectProc = "invisible";
        ProcSubset = "pid";

        # Privileges
        NoNewPrivileges = true;
        CapabilityBoundingSet =
          if cfg.port < 1024
          then "CAP_NET_BIND_SERVICE"
          else "";
        RestrictSUIDSGID = true;
        LockPersonality = true;
        RemoveIPC = true;
        KeyringMode = "private";

        # Devices
        PrivateDevices = true;
        DevicePolicy = "closed";

        # Network
        RestrictAddressFamilies = [
          "AF_INET"
          "AF_INET6"
          "AF_UNIX"
        ];
        SocketBindAllow = "tcp:${toString cfg.port}";

        # Syscalls
        SystemCallFilter = [
          "@system-service"
          "~@privileged"
          "~@resources"
        ];
        SystemCallArchitectures = "native";

        # Other :D
        MemoryDenyWriteExecute = true;
        RestrictNamespaces = true;
        RestrictRealtime = true;
      };
    };
  };
}
