{
  lib,
  testers,
  tuigreet-pkg,
  sway,
  kbd,
}:
testers.nixosTest {
  name = "tuigreet";
  meta.maintainers = with lib.maintainers; [NotAShelf];

  nodes = {
    machine = {
      users.users.alice = {
        isNormalUser = true;
        description = "Test User";
        password = "test123";
      };

      environment.systemPackages = [
        tuigreet-pkg
        sway
        kbd # for openvt
      ];

      services.greetd = {
        enable = true;
        settings = {
          terminal.vt = 1;
          default_session = {
            command = "${tuigreet-pkg}/bin/tuigreet --greeting 'Welcome to tuigreet!' --time --cmd sway";
            user = "greeter";
          };
        };
      };

      # Create a minimal wayland session for testing
      environment.etc."wayland-sessions/sway.desktop".text = ''
        [Desktop Entry]
        Name=Sway
        Comment=An i3-compatible Wayland compositor
        Exec=sway
        Type=Application
      '';

      # Create cache directory for tuigreet remember features
      systemd.tmpfiles.rules = [
        "d /var/cache/tuigreet 0755 greeter greeter -"
      ];
    };
  };

  testScript = ''
    machine.wait_for_unit("greetd.service")
    machine.wait_for_unit("getty@tty1.service")

    # Check that greetd and tuigreet are running
    machine.succeed("pgrep -f greetd")
    machine.succeed("pgrep -f tuigreet")

    # --list-outputs must exit 0. It prints a "no devices found" notice when
    # the VM has no DRM connectors, but must never fail.
    machine.succeed("tuigreet --list-outputs 2>&1")

    # [[outputs]] + [terminal] config must parse and validate cleanly.
    machine.succeed(
      "printf '[[outputs]]\\nconnector = \"Virtual-1\"\\nprimary = true\\n\\n[terminal]\\ncols = 200\\nrows = 60\\n'"
      " > /tmp/test-outputs.toml"
    )
    machine.succeed(
      "tuigreet --config /tmp/test-outputs.toml --dump-config > /tmp/dump.txt"
    )
    machine.succeed("grep -q 'cols = 200' /tmp/dump.txt")
    machine.succeed("grep -q 'rows = 60' /tmp/dump.txt")

    # Verify that [terminal] cols/rows is applied via TIOCSWINSZ.
    #
    # --dump-config causes tuigreet to apply terminal sizing and exit before
    # the TUI or any greetd connection is established.  Running via openvt
    # puts stdout on a real VT so the ioctl succeeds; stty then reads it back.
    machine.succeed(
      "printf '[terminal]\\ncols = 200\\nrows = 60\\n' > /tmp/test-terminal.toml"
    )
    machine.succeed(
      "openvt -c 8 -w -- tuigreet --config /tmp/test-terminal.toml --dump-config"
    )
    size = machine.succeed("stty -F /dev/tty8 size").strip()
    assert size == "60 200", f"Expected terminal size '60 200', got '{size}'"
  '';
}
