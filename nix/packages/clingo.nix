{ lib
, stdenv
, fetchFromGitHub
, cmake
, python3
,
}:
stdenv.mkDerivation rec {
  pname = "clingo";
  version = "5.6.2";

  src = fetchFromGitHub {
    owner = "potassco";
    repo = "clingo";
    rev = "v${version}";
    sha256 = "sha256-2vOscD5jengY3z9gHoY9y9y6RLfdzUj7BNKLyppNRac=";
  };

  nativeBuildInputs = [ cmake python3 ];

  cmakeFlags = [ "-DCLINGO_BUILD_WITH_PYTHON=ON" ];

  meta = {
    description = "ASP system to ground and solve logic programs";
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.raskin ];
    platforms = lib.platforms.unix;
    homepage = "https://potassco.org/";
    downloadPage = "https://github.com/potassco/clingo/releases/";
  };
}
