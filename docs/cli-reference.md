# CLI reference

This file is generated from the compiled CLI help. Do not edit it manually. Regenerate it with `node scripts/generate-cli-reference.mjs`.

## Top-level help

```text
Headless command-line interface for the Imaguncula icon generator

Usage: imaguncula <COMMAND>

Commands:
  export  Export an existing .imaguncula.json project
  quick   Export one source through one bundled preset without saving a project
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Saved-project export

```text
Export an existing .imaguncula.json project

Usage: imaguncula export [OPTIONS] --output <DIRECTORY> <PROJECT>

Arguments:
  <PROJECT>
          Project file ending in .imaguncula.json

Options:
      --allow-external-sources
          Read sources the project references outside its own directory.
          
          Project files are portable, so sources reached through absolute paths or .. segments are refused until this reviewed authorization is given.

  -o, --output <DIRECTORY>
          Parent directory that will receive <base>-icons; created when missing

      --overwrite
          Replace an existing final package directory

      --no-zip
          Do not include the deterministic combined ZIP

      --json
          Print the complete export result as JSON

  -h, --help
          Print help (see a summary with '-h')
```

## One-shot preset export

```text
Export one source through one bundled preset without saving a project

Usage: imaguncula quick [OPTIONS] --preset <PRESET_ID> --output <DIRECTORY> <SOURCE>

Arguments:
  <SOURCE>  SVG or PNG master source

Options:
  -p, --preset <PRESET_ID>  Exact bundled preset identifier, such as windows-desktop or web-pwa
      --name <NAME>         Project/display name; defaults to the source filename stem
      --role <ROLE=PATH>    Assign an additional preset source role, for example --role foreground=layer.svg
  -o, --output <DIRECTORY>  Parent directory that will receive <base>-icons; created when missing
      --overwrite           Replace an existing final package directory
      --no-zip              Do not include the deterministic combined ZIP
      --json                Print the complete export result as JSON
  -h, --help                Print help
```
