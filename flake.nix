{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };

  };
  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:

    let
      pkgs = import nixpkgs {
        inherit system;
      };

      frontend = 
       pkgs.mkYarnPackage {
          pname = "werwolf-frontend";
          src = ./web;

          configurePhase = ''
            cp -r $node_modules node_modules
            chmod -R 755 node_modules;
          '';

          buildPhase = ''
            yarn build
          '';

          installPhase = ''
            cp -r dist/  $out
          '';

          # do not attempt to build distribution bundles
          distPhase = ''
            true
          '';
        };
      
      backend = (let
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource (craneLib.path ./server);
        cargoArtifacts = craneLib.buildDepsOnly { inherit src; };
        bin = craneLib.buildPackage { inherit src cargoArtifacts;
          STATIC_PATH="${frontend}";
         };

        in 
          bin
        );

      
      in
        {
          packages = rec {
            default = docker;
            inherit backend frontend;

            docker = pkgs.dockerTools.buildImage {
              name = "werwolf";
              tag = "latest";
              copyToRoot = [ backend frontend ];
              config = {
                Cmd = [ "${backend}/bin/werwolf" ];
              };
          };
          };
        }
    );
      
}