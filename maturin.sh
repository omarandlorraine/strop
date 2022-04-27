#!/bin/bash

cd stroppy
python -m venv .env
source .env/bin/activate
pip install maturin
maturin develop

