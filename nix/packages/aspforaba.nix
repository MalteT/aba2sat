{ fetchFromBitbucket
, python3
, clingo
,
}:
let
  name = "ASPforABA";
  version = "ICCMA23";
  src = fetchFromBitbucket {
    owner = "coreo-group";
    repo = name;
    rev = "591d488";
    hash = "sha256-QdcisBOsGPOq9/KCQAUpKzQS7E2Olg4Zv+0jmf3/GkU=";
  };
  clingoWithPython = clingo.overrideAttrs (old: {
    cmakeFlags = [ "-DCLINGO_BUILD_WITH_PYTHON=ON" ];
    nativeBuildInputs = old.nativeBuildInputs ++ [ python3 ];
  });
in
python3.pkgs.buildPythonPackage {
  inherit src version;
  pname = name;

  format = "other";

  pythonPath = [ clingoWithPython python3.pkgs.cffi ];

  makeWrapperArgs = [ "--run 'mkdir -p /tmp/clingo'" ];

  patchPhase = ''
    rm ./configure
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp aspforaba.py $out/bin/ASPforABA
    cp -r encodings $out/bin/

    cat << EOF > $out/bin/.config
    TEMP_PATH=/tmp/clingo
    CLINGO_PATH=${clingoWithPython}/bin/clingo
    EOF
  '';
}
