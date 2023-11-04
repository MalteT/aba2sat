{
  rustPlatform,
  lib,
}: let
  config = lib.trivial.importTOML ../../Cargo.toml;
in
  rustPlatform.buildRustPackage {
    pname = config.package.name;
    version = config.package.version;

    src = lib.sources.cleanSource ../..;

    cargoDeps = {
      lockFile = "../../Cargo.lock";
    };

    cargoLock = {
      lockFile = ../../Cargo.lock;
    };

    meta = {
      # description = "Awesome program description";
      # homepage = "https://your.new.homepage";
      # license = licenses.mit;
      # maintainers = [];
    };
  }
