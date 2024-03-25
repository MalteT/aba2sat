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

run_dc_co() {
  if [ -z "$ADDITIONAL_ARG" ]; then
    print_help_and_exit "Parameter --arg is missing!"
  fi
  if [ -z "$ABA_FILE" ]; then
    print_help_and_exit "Parameter --file is missing!"
  fi
  printf "%40s " "$(basename "$ABA_FILE")"
  TIMEFORMAT='{"wall":"%E","system":"%S","user":"%U"}'
  our_result=$(command time -f "$TIMEFORMAT" -o /tmp/aba2sat-time "$ABA2SAT" --file "$ABA_FILE" dc-co --query "$ADDITIONAL_ARG")
  other_result=$(command time -f "$TIMEFORMAT" -o /tmp/aspforaba-time "$ASPFORABA" --file "$ABA_FILE" --problem DC-CO --query "$ADDITIONAL_ARG")
  if [ "$our_result" != "$other_result" ]; then
    printf "❌\n"
    printf "%40s:%s\n" "arg" "$ADDITIONAL_ARG"
    printf "%40s:%40s %s\n" "Ours" "$our_result" "$(jq --compact-output --color-output </tmp/aba2sat-time)"
    printf "%40s:%40s %s\n" "Theirs" "$other_result" "$(jq --compact-output --color-output </tmp/aspforaba-time)"
  else
    printf "✅\n"
    printf "%40s:%40s %s\n" "Ours" "$our_result" "$(jq --compact-output --color-output </tmp/aba2sat-time)"
    printf "%40s:%40s %s\n" "Theirs" "$other_result" "$(jq --compact-output --color-output </tmp/aspforaba-time)"
  fi
}

POSITIONAL_ARGS=()
ASPFORABA=ASPforABA
ABA2SAT=result/bin/aba2sat
ABA_FILE=
ABA_FILE_DIR=
ABA_FILE_EXT=aba
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
