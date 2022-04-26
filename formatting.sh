#!/bin/bash

PROBLEM=0
for i in $(git ls-files | grep \\.rs$); do
	RE=$(rustfmt --write-mode diff $i 2>/dev/null || true)
	if [ -z "$RE" ]; then
		echo Formatting problem in $i
		PROBLEM=1
	fi
done

exit $PROBLEM

