#!/bin/bash
echo "Starting test"
docker compose down
docker compose up --exit-code-from ubuntu
if [ $? -ne 0 ]; then
    echo "Test failed"
    exit 1
else
    echo "Test was successful"
    exit 0
fi