#!/bin/sh
set -e

# This is more simple as a shell script than directly
# in the Makefile due to python venv. It needs to activate and deactivate

# Required system dependencies
sudo apt-get install devscripts apt-utils -y

# Activate the virtual environment first
python3 -m venv ./.venv
. ./.venv/bin/activate

# Install Nala dependencies
python3 -m pip install pyinstaller
python3 -m pip install ./
poetry install

# make sure directories are clean
rm -rf ./build/ ./dist/ ./**/__pycache__/

# Get the venv paths list
venv_paths=$(python3 -c '
from site import getsitepackages
from os import path, curdir

paths = getsitepackages()
args = map(lambda pth: f"--paths {path.relpath(pth, start = curdir)}", paths)
print(" ".join(args))
')

# Get the system paths list
system_paths=$(sudo python3 -c '
from site import getsitepackages

paths = getsitepackages()
args = map(lambda pth: f"--paths {pth}", paths)
print(" ".join(args))
')

# Get the excluded modules
excludes=$(python3 -c '
excludes = [
    "IPython",
    "IPython.display",
    "IPython.core",
    "IPython.core.formatters",
    "ipywidgets",
    "java",
    "java.lang",
    "winreg",
    "_winreg",
    "_winapi",
    "win32api",
    "win32com",
    "win32com.shell",
    "msvcrt",
]

args = map(lambda exclude: f"--exclude-module {exclude}", excludes)
print(" ".join(args))
')

# The binary name
# This name should be unique among the folder/file names due to the Linux requirement
# For example, setting this to `nala` will result in errors due to the `nala`` folder name in the same directory
binary_name="nala-cli"

pyinstaller --noconfirm \
    --clean \
    --console --nowindowed --noupx \
    $venv_paths \
    $system_paths \
    $excludes \
    --collect-all nala \
    --name $binary_name \
    ./nala-cli.py

# Remove the excluded modules from the warnings list
sed -i '/excluded module /d' ./build/$binary_name/warn-$binary_name.txt

# Add nala binary
mkdir ./dist/nala
mv ./dist/$binary_name ./dist/nala/$binary_name
echo '#!/bin/bash
nala_dir=$(dirname $(realpath $0))
$nala_dir/nala-cli/nala-cli $@
' >>./dist/nala/nala
chmod +x ./dist/nala/nala

# Archive the build and deactivate the virtual env
cd ./dist && tar cv nala/ | xz -9 >./nala.tar.xz && cd ../
deactivate

# Smoke test
./dist/nala/nala --help

# TODO add docs to the pyinstaller
# --add-data="README.rst:." \
# --add-data="docs:docs" \
