#!/bin/bash

DEFAULT='\e[0m'
RED='\e[41m'

# delete the app

rm ./bin/app
rm sealed_counter_state.bin

# build
make

if [ $? -ne 0 ]; then

   echo ""

   echo -e "${RED} make FAILED ${DEFAULT}"

   echo ""

   exit 0

fi
