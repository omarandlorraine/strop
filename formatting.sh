#!/bin/bash

PROBLEM=0
for i in $(git ls-files | grep \\.rs$); do
	if [ rustfmt --check $i 2>/dev/null ]; then
		echo Formatting problem in $i
		PROBLEM=1
	fi
done

exit $PROBLEM

