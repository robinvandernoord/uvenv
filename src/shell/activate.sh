function uvenv() {
  subcommand=$1
  venv_name=$2

  if [ "$subcommand" == "activate" ]; then
    # todo: eval uvenv activate ?
    if [ -z "$venv_name" ]; then
      echo "Error: No virtual environment name provided."
      return 1
    elif [ ! -d "$HOME/.local/uvenv/venvs/$venv_name" ]; then
      echo "Error: Virtual environment '$venv_name' does not exist."
      return 2
    else
      source "$HOME/.local/uvenv/venvs/$venv_name/bin/activate"
    fi
  else
    command uvenv "$@"
  fi
}
