#!/bin/bash

# filter the benchmark results with `15` and check if the number of lines is less than the original
[ $(cargo bench | wc -l) -gt $(cargo bench 15 | wc -l) ]

