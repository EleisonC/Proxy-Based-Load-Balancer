#!/bin/bash

URL="http://localhost:4000/work"

# Create a sequence of 40 numbers and pipe it to xargs
seq 1 40 | xargs -n1 -P40 -I {} curl -s -o /dev/null -w "Request {}: %{http_code}\n" $URL
