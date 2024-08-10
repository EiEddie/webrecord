#! /bin/bash

cd $(dirname $0)

test_path='../assets/test'
if [ ! -d $test_path ]; then
    mkdir $test_path
fi

sqlite3 $test_path/test.sqlite < table.sql
