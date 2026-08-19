{
  lib,
  craneLib,
  installShellFiles,
  versionCheckHook,
  scdoc,
}: let
  commonArgs = {
    pname = "tuigreet";
    version = (lib.importTOML ../Cargo.toml).workspace.package.version;
    src = let
      s = ../.;
      fs = lib.fileset;
    in
      fs.toSource {
        root = s;
        fileset = fs.unions [
          (s + /contrib)
          (s + /crates)
          (s + /xtask)
          (s + /Cargo.lock)
          (s + /Cargo.toml)
        ];
      };
    strictDeps = true;
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {name = "tuigreet-deps";});

  xtask = craneLib.buildPackage (commonArgs // {
    pname = "tuigreet-xtask";
    cargoArtifacts = null;
    cargoExtraArgs = "-p xtask";
    # Don't install the xtask binary — it's a build tool only.
    installPhaseCommand = "mkdir -p $out";
  });
in
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;
      useNextest = true;

      nativeInstallCheckInputs = [versionCheckHook];
      doInstallCheck = true;

      nativeBuildInputs = [
        installShellFiles
      ];

      postInstall = ''
        ${xtask}/bin/xtask man
        installManPage contrib/man/tuigreet.1
      '';

      meta = {
        description = "Stylish graphical console greeter for greetd";
        license = lib.licenses.gpl3Only;
        maintainers = with lib.maintainers; [NotAShelf];
        mainProgram = "tuigreet";
      };
    })
