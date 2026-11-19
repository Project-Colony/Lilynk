# Does the NixOS module actually evaluate, and does it still render the drop-ins we decided on?
# The units under them are the packages' own (scripts/*.service, packaging/linux/*.service);
# vm-test.nix runs them.
#
# WHY THIS FILE EXISTS. `nix flake check` does NOT evaluate `nixosModules`. Nix 2.24+ only forces
# the value, and Determinate Nix 3.23 (CI's) only checks it is a function or an attribute set.
# MEASURED: a flake whose `nixosModules.default` sets a nonexistent OPTION, references a
# nonexistent `pkgs` attribute AND calls a nonexistent `lib` function passes clean, printing the
# thoroughly reassuring
#
#     checking NixOS module 'nixosModules.default'... all checks passed!
#
# So every option name, type, `pkgs.*` reference and systemd directive in nixos-module.nix would
# be unverified by CI while reading as covered.
#
# HOW IT CLOSES THAT. Exposed as a flake `checks` output, so `nix flake check --no-build` — the leg
# CI already runs — must INSTANTIATE it, and instantiating forces the `assert` below. Every
# assertion is therefore pure Nix, evaluated at instantiation: a shell script inside the derivation
# would only run under a full `nix flake check`, which builds the hour-long Rust packages and is
# exactly what CI cannot afford. Keep it that way — if you add a check, add it to `results`, not to
# a `runCommand` body.
#
# STUB PACKAGES, on purpose. The real derivations would drag lilynk-host, lilynk-client and
# (via `gamescopeHdr`) a from-source gamescope into this check's closure, making the cheap leg
# expensive and coupling a module regression to a Rust build. What is under test here is the MODULE.
# ⚠ The stubs must be fake DERIVATIONS, not store-path strings: `types.package` accepts anything
# `isDerivation` as-is, but runs a store-path string through `builtins.storePath`, which demands the
# path actually exist and fails eval with "no substituter can build it".
{
  lib,
  runCommand,
  # The nixpkgs SOURCE. Interpolated rather than `nixpkgs + "/..."` because this is called with the
  # flake INPUT, which is an attribute set (coerced through its `outPath`) and not a path — the
  # difference only shows up as "expected a set but found a string" from whichever side is wrong.
  nixpkgs,
  system,
  module,
}:
let
  fakeDrv = name: {
    type = "derivation";
    inherit name;
    outPath = "/pf-stub/${name}";
    outputs = [ "out" ];
  };

  stubPackages =
    prefix:
    lib.genAttrs [
      "lilynk-host"
      "lilynk-client"
      "lilynk-web"
      "lilynk-scripting"
      "lilynk-seats"
      "lilynk-gamescope"
    ] (name: fakeDrv "${prefix}${name}");

  # `system-` marks the packages built from the system's nixpkgs.
  stubSelf = {
    packages.${system} = stubPackages "";
    lib.packagesWith = _: stubPackages "system-";
  };

  # A machine just complete enough for eval-config, plus the scenario under test.
  evalWith =
    scenario:
    (import "${nixpkgs}/nixos/lib/eval-config.nix" {
      inherit system;
      modules = [
        (module stubSelf)
        {
          boot.loader.grub.enable = false;
          fileSystems."/" = {
            device = "/dev/sda1";
            fsType = "ext4";
          };
          system.stateVersion = "24.11";
          nixpkgs.hostPlatform = system;
          # `host.users` adds group membership to an existing user; declare one so NixOS's own
          # "isNormalUser or isSystemUser" assertion is not what this check trips over.
          users.users.alice.isNormalUser = true;
        }
        scenario
      ];
    }).config;

  # Every scenario whose units are read keeps `gamescopeHdr = false`: its default would pull a
  # real (stub, here) gamescope onto the host unit's PATH, and nothing below is about that.
  # `scripting` is left at its default (on with the host) precisely BECAUSE the runner belongs
  # on that PATH — see the "not just in systemPackages" check.
  desktop = evalWith {
    services.lilynk.host = {
      enable = true;
      users = [ "alice" ];
      # Explicit: gamestream defaults to false (opt-in on every route) — this fixture
      # is the one that proves opting IN still reaches the argv and the firewall.
      gamestream = true;
      gamescopeHdr = false;
      desktopSession = true;
    };
  };

  appliance = evalWith {
    services.lilynk.host = {
      enable = true;
      autoStart = true;
      openFirewall = true;
      gamescopeHdr = false;
    };
  };

  nativeOnly = evalWith {
    services.lilynk.host = {
      enable = true;
      openFirewall = true;
      gamestream = false;
      gamescopeHdr = false;
    };
  };

  # A host that has opted out of the runner — the negative half of the PATH check.
  noScripting = evalWith {
    services.lilynk.host = {
      enable = true;
      gamescopeHdr = false;
    };
    services.lilynk.scripting.enable = false;
  };

  clientOnly = evalWith { services.lilynk.client.enable = true; };

  # The Steam Frame adapter hook rides on NetworkManager, which `desktop` leaves off.
  networkManager = evalWith {
    services.lilynk.host = {
      enable = true;
      gamescopeHdr = false;
    };
    networking.networkmanager.enable = true;
  };

  # The seat supervisor, on a host. The other two are the refusals: no host to run in each seat,
  # and users the supervisor could not create with useradd.
  seats = evalWith {
    services.lilynk.host = {
      enable = true;
      users = [ "alice" ];
      openFirewall = true;
      gamescopeHdr = false;
      settings.RUST_LOG = "info";
      settings.LILYNK_MGMT_BIND = "0.0.0.0:47991";
    };
    services.lilynk.web.bind = "127.0.0.1";
    services.lilynk.seats.enable = true;
  };
  seatsWithoutHost = evalWith { services.lilynk.seats.enable = true; };
  seatsImmutableUsers = evalWith {
    services.lilynk.host.enable = true;
    services.lilynk.seats.enable = true;
    users.mutableUsers = false;
  };

  unit = cfg: name: cfg.systemd.user.units."${name}.service".text;
  has =
    cfg: name: infix:
    lib.hasInfix infix (unit cfg name);
  # The same for a system unit, which a `systemd.services` entry renders under `systemd.units`.
  hasSys =
    cfg: name: infix:
    lib.hasInfix infix cfg.systemd.units."${name}".text;
  # A module's own failed assertions, as messages.
  failedAssertions = cfg: map (a: a.message) (lib.filter (a: !a.assertion) cfg.assertions);
  # A shipped unit's EnvironmentFile entries without host.env, which NixOS never reads. A drop-in
  # that clears the list must restate these, in this order.
  shippedEnvFiles =
    unitFile:
    lib.filter (f: !(lib.hasSuffix "/host.env" f)) (
      map (lib.removePrefix "EnvironmentFile=") (
        lib.filter (lib.hasPrefix "EnvironmentFile=") (lib.splitString "\n" (builtins.readFile unitFile))
      )
    );

  results = [
    # --- the module evaluates at all, in every shape an operator can ask for -------------------
    {
      name = "desktop scenario has no failing assertions";
      ok = failedAssertions desktop == [ ];
    }
    {
      name = "appliance scenario has no failing assertions";
      ok = failedAssertions appliance == [ ];
    }
    {
      name = "client-only scenario has no failing assertions";
      ok = failedAssertions clientOnly == [ ];
    }
    {
      # `drvPath` instantiates the wrapper, so every package it names must exist.
      name = "the Steam Frame adapter hook reaches NetworkManager, and only with it";
      ok =
        let
          hooks = networkManager.networking.networkmanager.dispatcherScripts;
        in
        failedAssertions networkManager == [ ]
        && lib.length hooks == 1
        && lib.hasInfix "90-lilynk-frame-adapter" (lib.head hooks).source.drvPath
        && desktop.networking.networkmanager.dispatcherScripts == [ ];
    }

    # --- user scoping: the second-copy-steals-the-ports trap -----------------------------------
    # `systemd.user.*` installs into EVERY user's manager, root's included (user@0.service exists
    # as soon as anyone logs in as root), and `autoStart` puts these in default.target. Root's host
    # then wins the fixed ports and the desktop user's restarts forever on
    # `bind RTSP 48010: Address already in use` — every other listener in its log having bound
    # fine, so it reads like an unrelated program.
    {
      # `|` = TRIGGERING condition, which systemd ORs. Plain repeated ConditionUser= lines are
      # ANDed and would match nobody — the whole reason the prefix is there.
      name = "host.users scopes every user unit to those users, OR-ed";
      ok =
        let
          scoped = name: has desktop name "ConditionUser=|alice";
        in
        scoped "lilynk-host"
        && scoped "lilynk-web"
        && scoped "lilynk-web-init"
        && scoped "lilynk-scripting";
    }
    {
      # With no host.users to name, still keep SYSTEM users (root) out, while leaving the module
      # header's manual `systemctl --user enable --now lilynk-host` working for a normal login.
      name = "with no host.users, the units still refuse system users (root)";
      ok =
        has appliance "lilynk-host" "ConditionUser=!@system"
        && !(has appliance "lilynk-host" "ConditionUser=|");
    }

    # --- the KWin identification trap (packaging/arch/lilynk-host.install) -------------------
    # The host MUST exec the plain store path. A capability wrapper here would put CAP_SYS_NICE in
    # the process's permitted set, and the kernel then refuses KWin the /proc/<pid>/exe readlink it
    # identifies the client by. The packaged ExecStart is that path; GameStream replaces it here.
    {
      # The empty line clears the packaged ExecStart: two on a simple service fail the unit.
      name = "gamestream=true runs the store binary with --gamestream, never a wrapper";
      ok =
        has desktop "lilynk-host"
          "ExecStart=\nExecStart=/pf-stub/system-lilynk-host/bin/lilynk-host serve --gamestream\n"
        && !(has desktop "lilynk-host" "ExecStart=/run/wrappers");
    }
    # ...while the ENCODE WORKER, which nothing ever has to identify, is pointed at the wrapper.
    {
      name = "host points LILYNK_ENCODE_WORKER at the capability wrapper";
      ok =
        has desktop "lilynk-host"
          "LILYNK_ENCODE_WORKER=/run/wrappers/bin/lilynk-encode-worker";
    }
    {
      name = "the encode-worker wrapper carries exactly cap_sys_nice=ep";
      ok = desktop.security.wrappers.lilynk-encode-worker.capabilities == "cap_sys_nice=ep";
    }
    {
      name = "only the host's users run the encode-worker wrapper";
      ok =
        let
          w = desktop.security.wrappers.lilynk-encode-worker;
        in
        w.permissions == "u+rx,g+rx"
        && w.group == "lilynk"
        && seats.security.wrappers.lilynk-encode-worker.group == "lilynk-games"
        && lib.elem "lilynk-games" seats.users.users.alice.extraGroups;
    }
    {
      # A seat on a person's own account joins no group of ours; its unit carries the wrapper's.
      name = "every seat's unit carries the encode-worker wrapper's group";
      ok = hasSys seats "lilynk-seat@.service" "SupplementaryGroups=lilynk-games";
    }

    # --- the desktop-login route (scripts/lilynk-host-desktop-session.conf) -----------------
    # Asserted on the evaluated LISTS, not the rendered text: systemd renders `After=` as one
    # space-separated line, so `hasInfix "After=graphical-session.target"` silently depends on
    # ordering.
    {
      name = "desktopSession binds the host to graphical-session.target";
      ok =
        let
          u = desktop.systemd.user.services.lilynk-host;
        in
        lib.elem "graphical-session.target" u.after
        && lib.elem "graphical-session.target" u.partOf
        # IN ADDITION to default.target, never instead of it.
        && lib.elem "graphical-session.target" u.wantedBy;
    }
    {
      name = "desktopSession is NOT applied to the appliance route (it would stay stopped there)";
      ok =
        let
          u = appliance.systemd.user.services.lilynk-host;
        in
        !(lib.elem "graphical-session.target" u.partOf) && lib.elem "default.target" u.wantedBy;
    }

    # --- GameStream opt-out reaches both the argv and the firewall -----------------------------
    {
      name = "gamestream=false drops --gamestream and its firewall ports";
      ok =
        !(has nativeOnly "lilynk-host" "--gamestream")
        && !(lib.elem 47984 nativeOnly.networking.firewall.allowedTCPPorts)
        && lib.elem 47990 nativeOnly.networking.firewall.allowedTCPPorts;
    }
    {
      # The DEFAULT is the secure native-only host — gamestream unset must behave like
      # gamestream=false (opt-in on every package route; the appliance fixture leaves it unset).
      name = "gamestream default (unset) is native-only";
      ok =
        !(has appliance "lilynk-host" "--gamestream")
        && !(lib.elem 47984 appliance.networking.firewall.allowedTCPPorts);
    }
    {
      name = "openFirewall opens the console AND its plugin origin";
      ok =
        lib.elem 47992 appliance.networking.firewall.allowedTCPPorts
        && lib.elem 47993 appliance.networking.firewall.allowedTCPPorts;
    }

    # --- the console ---------------------------------------------------------------------------
    {
      # web-init.sh writes a LILYNK_UI_BIND into host.env; read, it would beat `web.bind`. The
      # rest is the shipped list as it stands, mandatory token included.
      name = "the console binds where web.bind says, never host.env";
      ok =
        let
          files = appliance.systemd.user.services.lilynk-web.serviceConfig.EnvironmentFile;
        in
        lib.head files == ""
        && lib.tail files == shippedEnvFiles ../../scripts/lilynk-web.service
        && has appliance "lilynk-web" "LILYNK_UI_BIND=0.0.0.0";
    }

    # --- the packaged units and the runner's drop-in --------------------------------------------
    {
      # The module's units are drop-ins: without the packages there is no unit to extend.
      name = "the host, console and runner units come from their packages";
      ok =
        let
          p = appliance.services.lilynk;
        in
        lib.all (pkg: lib.elem pkg appliance.systemd.packages) [
          p.host.package
          p.web.package
          p.scripting.package
        ];
    }
    {
      # Without bwrap on its PATH the runner starts no plugin at all, and the library is empty.
      name = "the plugin runner can build a sandbox";
      ok = builtins.any (p: lib.hasInfix "bubblewrap" (toString p)) (
        appliance.systemd.user.services.lilynk-scripting.path or [ ]
      );
    }
    {
      # A network plugin's namespace comes from `unshare`, and pasta connects it. Without either
      # on PATH that plugin does not start; a seat's runner runs network plugins too.
      name = "the plugin runner can connect a network plugin";
      ok =
        lib.all
          (
            unit:
            lib.all (pkg: builtins.any (p: lib.hasInfix pkg (toString p)) (unit.path or [ ])) [
              "passt"
              "util-linux"
            ]
          )
          [
            appliance.systemd.user.services.lilynk-scripting
            appliance.systemd.user.services."lilynk-scripting-seat@"
          ];
    }
    {
      # The library scanners are plugins, so a runner that is off means an EMPTY LIBRARY and
      # no obvious reason why — which is why deb+rpm `systemctl --global enable` it.
      name = "the plugin runner is started by default, like every other channel";
      ok = has appliance "lilynk-scripting" "WantedBy=default.target";
    }
    {
      # The runner has to be on the HOST unit's PATH, not merely in systemPackages. Package ops
      # (`plugins add`, and the console's store jobs, which run inside the host process) locate
      # `lilynk-scripting` as an executable, and on NixOS PATH is the only rung that can ever
      # match — the runner is its own derivation, so it is never beside the host binary and never
      # under /usr. systemPackages covers an operator's shell and NOT this unit, so without the
      # PATH entry the console reports a running, enabled runner as "not installed".
      name = "the plugin runner is on the host unit's PATH, not just in systemPackages";
      ok =
        has appliance "lilynk-host" "/pf-stub/lilynk-scripting/bin"
        && has desktop "lilynk-host" "/pf-stub/lilynk-scripting/bin";
    }
    {
      name = "the host unit PATH includes the system profile so nested steam resolves";
      ok = has desktop "lilynk-host" "system-path/bin";
    }
    {
      # The runner is on that PATH only when it is installed, so `scripting.enable = false` does
      # not put a package the machine never built onto a unit's PATH.
      name = "a host without the runner does not carry it on PATH";
      ok = !(has noScripting "lilynk-host" "/pf-stub/lilynk-scripting/bin");
    }

    # --- a switch that changes a package restarts the running user services --------------------
    {
      name = "a host package change reloads the unit that restarts the user services";
      ok =
        let
          u = desktop.systemd.services.lilynk-restart-user-units;
        in
        lib.elem desktop.services.lilynk.host.package u.reloadTriggers
        && lib.hasInfix "restart-user-units" u.serviceConfig.ExecReload;
    }

    # --- the seat supervisor -------------------------------------------------------------------
    {
      name = "the seat scenario has no failing assertions";
      ok = failedAssertions seats == [ ];
    }
    {
      # Both units and the tmpfiles rules come from the package; the module only adds what a
      # packaged unit lacks.
      name = "seats installs the package's units and tmpfiles rules";
      ok =
        lib.elem seats.services.lilynk.seats.package seats.systemd.packages
        && lib.elem seats.services.lilynk.seats.package seats.systemd.tmpfiles.packages;
    }
    {
      # `enable` is the operator's decision here, so the daemon starts at boot. The packaged
      # distros ship it disabled and let the console turn it on.
      name = "seats starts the supervisor at boot";
      ok = hasSys seats "lilynk-seats.service" "WantedBy=multi-user.target";
    }
    {
      # A unit from `systemd.packages` has no PATH of its own, and useradd, setfacl and
      # loginctl are all looked up on it.
      name = "seats gives both units a PATH with the account tools";
      ok =
        let
          onPath = name: hasSys seats name "-shadow-" && hasSys seats name "-acl-";
        in
        onPath "lilynk-seats.service" && onPath "lilynk-seat@.service";
    }
    {
      # seat-session defaults to /usr/bin/lilynk-host, which NixOS has not got.
      name = "seats points seat-session at the system's host build";
      ok =
        hasSys seats "lilynk-seat@.service"
          "LILYNK_HOST_BIN=/pf-stub/system-lilynk-host/bin/lilynk-host";
    }
    {
      # The console turns the door on through the helper templates, which only this group starts.
      name = "seats put the host's users in lilynk-update, and only with seats on";
      ok =
        lib.elem "lilynk-update" seats.users.users.alice.extraGroups
        && !lib.elem "lilynk-update" desktop.users.users.alice.extraGroups;
    }
    {
      name = "a switch leaves running seats up";
      ok = hasSys seats "lilynk-seat@.service" "X-RestartIfChanged=false";
    }
    {
      # The owner's row runs its host from seat-session, so it takes the module's settings and
      # the gamestream flag from a file only the owner's branch reads, and every seat encodes
      # through the capped worker.
      name = "seats hand the owner's row the module's settings and every seat the capped worker";
      ok =
        let
          env = seats.systemd.services."lilynk-seat@".environment;
        in
        lib.hasSuffix "-lilynk-owner.env" env.LILYNK_OWNER_ENV
        && lib.hasSuffix "/lilynk-encode-worker" env.LILYNK_ENCODE_WORKER;
    }
    {
      name = "seats refuses a box with no host";
      ok = lib.any (lib.hasInfix "seats needs services.lilynk.host.enable") (
        failedAssertions seatsWithoutHost
      );
    }
    {
      name = "seats refuses immutable users";
      ok = lib.any (lib.hasInfix "seats needs users.mutableUsers = true") (
        failedAssertions seatsImmutableUsers
      );
    }
    {
      name = "a host without seats defines neither seat unit";
      ok =
        !(desktop.systemd.services ? lilynk-seats) && !(desktop.systemd.services ? "lilynk-seat@");
    }

    # --- the door: the box's host as a system service ------------------------------------------
    {
      # The packaged unit names /usr/bin/lilynk-host. The empty line first clears it: a second
      # ExecStart on a simple service is an error.
      name = "the door runs the system's host build";
      ok =
        hasSys seats "lilynk-door.service"
          "ExecStart=\nExecStart=/pf-stub/system-lilynk-host/bin/lilynk-host serve --door";
    }
    {
      name = "the door's console runs the module's web package";
      ok =
        hasSys seats "lilynk-web-door.service"
          "ExecStart=\nExecStart=/pf-stub/lilynk-web/bin/lilynk-web-server";
    }
    {
      # The door verb behind the helpers and the door's ExecStartPre calls useradd and runuser
      # by name.
      name = "the door's helpers have a PATH with the account tools";
      ok =
        let
          onPath =
            name:
            hasSys seats name "-shadow-"
            && hasSys seats name "-util-linux-"
            && hasSys seats name "-gawk-";
        in
        onPath "lilynk-door-on@.service"
        && onPath "lilynk-door-off@.service"
        && onPath "lilynk-door.service";
    }
    {
      name = "the door takes the port keys from settings and no other knob";
      ok =
        let
          env = seats.systemd.services.lilynk-door.environment;
        in
        env.LILYNK_MGMT_BIND or null == "0.0.0.0:47991" && !(env ? RUST_LOG);
    }
    {
      # The helper's `systemctl enable` has nowhere to write on NixOS, so the units are wired
      # here. They start at boot only while the owner record exists and no off holds them: a
      # failed off can't disable them.
      name = "the door's units start at boot with the owner record and no hold";
      ok =
        let
          wired =
            name: unit:
            let
              text = builtins.readFile unit;
            in
            seats.systemd.services.${name}.wantedBy == [ "multi-user.target" ]
            && lib.hasInfix "\nConditionPathExists=/var/lib/lilynk-seats/owner\n" text
            && lib.hasInfix "\nConditionPathExists=!/var/lib/lilynk-seats/door-hold\n" text;
        in
        wired "lilynk-door" ../linux/lilynk-door.service
        && wired "lilynk-web-door" ../linux/lilynk-web-door.service;
    }
    {
      # A console limited to this machine stays so when the door takes over. host.env, which
      # carries the bind on other distros, is not read: the option is the source.
      name = "the door's console binds where web.bind says, never host.env";
      ok =
        let
          files = seats.systemd.services.lilynk-web-door.serviceConfig.EnvironmentFile;
        in
        hasSys seats "lilynk-web-door.service" "LILYNK_UI_BIND=127.0.0.1"
        && lib.head files == ""
        && lib.tail files == shippedEnvFiles ../linux/lilynk-web-door.service;
    }
    {
      # polkit reads /etc/polkit-1/rules.d; the rule is the package's file, not a copy.
      name = "the door's polkit rule is the package's file";
      ok =
        seats.security.polkit.enable
        && seats.environment.etc."polkit-1/rules.d/49-lilynk-door.rules".source
          == "/pf-stub/lilynk-seats/share/polkit-1/rules.d/49-lilynk-door.rules";
    }
    {
      # The rule and the helper both name this group.
      name = "the door's group exists";
      ok = seats.users.groups ? lilynk-update;
    }
    {
      # The door's user has a group of its own. `lilynk` owns the usbip attach nodes and
      # passes ly-dm-helper, which the door has no use for.
      name = "the door's units run in a group the module declares";
      ok =
        seats.users.groups ? lilynk-door
        && lib.all (f: lib.hasInfix "\nGroup=lilynk-door\n" (builtins.readFile f)) [
          ../linux/lilynk-door.service
          ../linux/lilynk-web-door.service
        ];
    }
    {
      name = "seats open the seat ports with the host's";
      ok = lib.elem {
        from = 9779;
        to = 9786;
      } seats.networking.firewall.allowedUDPPortRanges;
    }
    {
      # The packaged list starts with host.env, which NixOS does not read. The owner row's env
      # comes after `settings`, so its ports and the box's identity win.
      name = "the host reads settings, never host.env, and the seat env last";
      ok =
        let
          files = seats.systemd.user.services.lilynk-host.serviceConfig.EnvironmentFile;
          shipped = shippedEnvFiles ../../scripts/lilynk-host.service;
        in
        lib.head files == ""
        && lib.length files == 3
        && lib.hasInfix "lilynk-host.env" (lib.elemAt files 1)
        && shipped == [ "-/run/lilynk/seats/%u.env" ]
        && lib.last files == lib.last shipped;
    }
    {
      name = "a host without seats keeps its ports and units as they were";
      ok =
        appliance.networking.firewall.allowedUDPPortRanges == [ ]
        && !(desktop.systemd.services ? lilynk-door)
        && !(desktop.users.groups ? lilynk-update)
        && !(desktop.users.groups ? lilynk-door);
    }

    # --- the client half must not drag the host's system wiring in -----------------------------
    {
      name = "a client-only machine defines no host/web/scripting units";
      ok =
        !(clientOnly.systemd.user.services ? lilynk-host)
        && !(clientOnly.systemd.user.services ? lilynk-web)
        && !(clientOnly.systemd.user.services ? lilynk-scripting)
        && !(clientOnly.systemd.services ? lilynk-restart-user-units);
    }

    # --- GPU-loading packages come from the system's nixpkgs -----------------------------------
    {
      name = "host, client and gamescope default to the system build; web and scripting do not";
      ok =
        let
          p = desktop.services.lilynk;
        in
        p.host.package.name == "system-lilynk-host"
        && p.host.gamescopePackage.name == "system-lilynk-gamescope"
        && clientOnly.services.lilynk.client.package.name == "system-lilynk-client"
        && p.web.package.name == "lilynk-web"
        && p.scripting.package.name == "lilynk-scripting"
        && p.seats.package.name == "lilynk-seats";
    }
  ];

  failures = map (r: r.name) (lib.filter (r: !r.ok) results);
in
# The `assert` is what makes `--no-build` sufficient: instantiating this derivation forces it.
assert
  failures == [ ]
  || throw ''
    The lilynk NixOS module no longer renders what packaging/nix/module-check.nix requires.
    Failing checks (${toString (lib.length failures)} of ${toString (lib.length results)}):
      - ${lib.concatStringsSep "\n    - " failures}
  '';
runCommand "lilynk-nixos-module-check"
  {
    # Recorded in the output so a green run says what it actually covered.
    passed = toString (lib.length results);
  }
  ''
    echo "lilynk NixOS module: $passed checks passed at eval time" > "$out"
  ''
