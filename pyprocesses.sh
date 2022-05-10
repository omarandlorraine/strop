#!/usr/bin/env bash

cd stroppy
python -m venv .env
source .env/bin/activate

case $1 in
    "setup")
        pip install maturin
        cargo build --verbose
        maturin develop
        ;;
    "lint")
        pip install flake8
        flake8 .././python_tests.py
        ;;
    "test")
        .././python_tests.py
        ;;
esac
