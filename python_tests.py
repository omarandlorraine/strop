#!/usr/bin/python3
"""
This source file is supposed to serve as an example of how to use the `stroppy`
module from Python 3, and also serves as basic tests in my CI/CD pipeline.

As for Python 2, I'm not even trying to support that.
"""

import sys
import stroppy

print(sys.version)

# Make sure these functions actually are a part of the module
stroppy.string_sum # pylint: disable=pointless-statement
stroppy.sum_as_string # pylint: disable=pointless-statement
