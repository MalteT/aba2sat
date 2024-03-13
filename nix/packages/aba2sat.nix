{ rustPlatform
, lib
,
}:
let
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
    description = "ABA solver using a SAT backend";
    # homepage = "https://your.new.homepage";
    license = lib.licenses.gpl3;
    # maintainers = [];
  };
}
