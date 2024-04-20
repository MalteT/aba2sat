#!/usr/bin/env bash

print_help_and_exit() {
  if [ -n "$1" ]; then
    printf "%s\n\n" "$1"
  fi
  printf "Usage: validate [OPTIONS] \n"
  printf "\n"
  printf "Options:\n"
  printf "  --aspforaba\n"
  printf "          Binary to use when calling aspforaba\n"
  printf "  -p, --problem\n"
  printf "          The problem to solve\n"
  printf "  -a, --arg\n"
  printf "          The additional argument for the problem\n"
  printf "  -f, --file\n"
  printf "          The file containing the problem in ABA format\n"
  printf "  --files-from\n"
  printf "          Use the following dir to read files, specify a single file with --file instead\n"
  exit 1
}

format_time() {
  COMMAND="$1"
  FILE="$2"
  mean=$(jq ".results[]   | select(.command == \"$COMMAND\") | (.mean   * 1000)" "$FILE")
  stddev=$(jq ".results[] | select(.command == \"$COMMAND\") | (.stddev * 1000)" "$FILE")
  printf "%7.3f±%7.3fms" "$mean" "$stddev"
}

run_dc_co() {
  OUTPUT_DIR=${OUTPUT_DIR:-$(mktemp -d)}
  mkdir -p "$OUTPUT_DIR"
  JSON_FILE="$OUTPUT_DIR/$(basename "$ABA_FILE")-hyperfine.json"
  # Restrict memory to 20GB
  ulimit -v 20000000
  if [ -z "$ADDITIONAL_ARG" ]; then
    print_help_and_exit "Parameter --arg is missing!"
  fi
  if [ -z "$ABA_FILE" ]; then
    print_help_and_exit "Parameter --file is missing!"
  fi
  printf "===== %s ==== " "$(basename "$ABA_FILE")"
  our_result=$("$ABA2SAT" --file "$ABA_FILE" dc-co --query "$ADDITIONAL_ARG" | tee "$OUTPUT_DIR/$(basename "$ABA_FILE")-aba2sat-result")
  other_result=$("$ASPFORABA" --file "$ABA_FILE" --problem DC-CO --query "$ADDITIONAL_ARG" | tee "$OUTPUT_DIR/$(basename "$ABA_FILE")-aspforaba-result")
  $HYPERFINE --shell=none \
    --export-json "$JSON_FILE" \
    --command-name "aba2sat" \
    "$ABA2SAT --file \"$ABA_FILE\" dc-co --query \"$ADDITIONAL_ARG\"" \
    --command-name "aspforaba" \
    "$ASPFORABA --file \"$ABA_FILE\" --problem DC-CO --query \"$ADDITIONAL_ARG\"" 1>/dev/null 2>&1
  if [ "$our_result" != "$other_result" ]; then
    printf "❌\n"
  else
    printf "✅\n"
  fi
  printf "Argument: %s\n" "$ADDITIONAL_ARG"
  printf "Our:      %3s %30s\n" "$our_result" "$(format_time "aba2sat" "$JSON_FILE")"
  printf "Their:    %3s %30s\n" "$other_result" "$(format_time "aspforaba" "$JSON_FILE")"
}

POSITIONAL_ARGS=()
ASPFORABA=ASPforABA
ABA2SAT=result/bin/aba2sat
ABA_FILE=
ABA_FILE_DIR=
ABA_FILE_EXT=aba
HYPERFINE=hyperfine
PROBLEM=
ADDITIONAL_ARG=

while [[ $# -gt 0 ]]; do
  case $1 in
  -h | --help)
    print_help_and_exit
    ;;
  --aspforaba)
    shift
    ASPFORABA=$1
    shift
    ;;
  -p | --problem)
    shift
    PROBLEM=$1
    shift
    ;;
  -f | --file)
    if [ -n "$ABA_FILE_DIR" ]; then
      print_help_and_exit "Parameters --file and --files-from cannot be mixed"
    fi
    shift
    ABA_FILE=$1
    shift
    ;;
  --files-from)
    if [ -n "$ABA_FILE" ]; then
      print_help_and_exit "Parameters --file and --files-from cannot be mixed"
    fi
    if [ -n "$ADDITIONAL_ARG" ]; then
      print_help_and_exit "Parameters --arg and --files-from cannot be mixed"
    fi
    shift
    ABA_FILE_DIR=$1
    shift
    ;;
  -a | --arg)
    if [ -n "$ABA_FILE_DIR" ]; then
      print_help_and_exit "Parameters --arg and --files-from cannot be mixed"
    fi
    shift
    ADDITIONAL_ARG=$1
    shift
    ;;
  --aba2sat)
    shift
    ABA2SAT=$1
    shift
    ;;
  -*)
    echo "Unknown option $1"
    print_help_and_exit
    ;;
  *)
    POSITIONAL_ARGS+=("$1") # save positional arg
    shift                   # past argument
    ;;
  esac
done

set -- "${POSITIONAL_ARGS[@]}" # restore positional parameters

case "$PROBLEM" in
dc-co | DC-CO)
  if [ -n "$ABA_FILE_DIR" ]; then
    # run for every file found in the directory
    for file in "$ABA_FILE_DIR"/*."$ABA_FILE_EXT"; do
      ABA_FILE="$file" ADDITIONAL_ARG="$(cat "$file.asm")" run_dc_co
    done
  else
    # run for the single configured file
    run_dc_co
  fi
  ;;
*)
  print_help_and_exit "Problem $PROBLEM is not supported"
  ;;
esac
