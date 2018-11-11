#!/bin/bash

branch =$(python aws-sdk-cpp/CI/ExtractBuildArgs.py branch)
git clone git@github.com:awslabs/aws-sdk-cpp-staging.git aws-sdk-cpp
cd aws-sdk-cpp
git reset --hard HEAD
git checkout master
git pull
git checkout $branch
